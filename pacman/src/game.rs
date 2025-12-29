//! This module contains the main game logic and state.

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

use std::collections::HashMap;
use std::ops::Not;
use tracing::{debug, info, trace, warn};

use crate::constants::{self, animation, MapTile, CANVAS_SIZE};
use crate::error::{GameError, GameResult};
use crate::events::{CollisionTrigger, GameEvent, StageTransition};
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::item::PelletCount;
use crate::systems::state::{IntroPlayed, PauseState};
use crate::systems::{
    self, audio_system, blinking_system, collision_system, combined_render_system, directional_render_system,
    dirty_render_system, eaten_ghost_system, fruit_sprite_system, ghost_collision_observer, ghost_movement_system,
    ghost_state_system, hud_render_system, item_collision_observer, linear_render_system, player_life_sprite_system,
    present_system, profile, time_to_live_system, touch_ui_render_system, AudioEvent, AudioResource, AudioState,
    BackbufferResource, Blinking, BufferedDirection, Collider, DebugState, DebugTextureResource, DeltaTime, DirectionalAnimation,
    EntityType, Frozen, FruitSprites, GameStage, Ghost, GhostAnimation, GhostAnimations, GhostBundle, GhostCollider, GhostState,
    GlobalState, ItemBundle, ItemCollider, LastAnimationState, LinearAnimation, MapTextureResource, MovementModifiers, NodeId,
    PacmanCollider, PlayerAnimation, PlayerBundle, PlayerControlled, PlayerDeathAnimation, PlayerLives, Position, RenderDirty,
    Renderable, ScoreResource, SystemId, SystemTimings, Timing, TouchState, Velocity, Visibility,
};

#[cfg(not(target_os = "emscripten"))]
use crate::systems::StartupSequence;

use crate::texture::animated::{DirectionalTiles, TileSequence};
use crate::texture::sprite::AtlasTile;
use crate::texture::sprites::{FrightenedColor, GameSprite, GhostSprite, MazeSprite, PacmanSprite};
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::event::EventRegistry;
use bevy_ecs::observer::Trigger;
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, SystemSet};
use bevy_ecs::system::{Local, Res, ResMut};
use bevy_ecs::world::World;
use sdl2::event::EventType;
use sdl2::image::LoadTexture;
use sdl2::render::{BlendMode, Canvas, ScaleMode, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::{
    asset::Asset,
    events::GameCommand,
    map::render::MapRenderer,
    platform,
    systems::{BatchedLinesResource, Bindings, CursorPosition, TtfAtlasResource},
    texture::sprite::{AtlasMapper, SpriteAtlas},
};

/// System set for all gameplay systems to ensure they run after input processing
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameplaySet {
    /// Gameplay systems that process inputs
    Input,
    /// Gameplay systems that update the game state
    Update,
    /// Gameplay systems that respond to events
    Respond,
}

/// System set for all rendering systems to ensure they run after gameplay logic
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum RenderSet {
    Animation,
    Draw,
    Present,
}

/// Core game state manager built on the Bevy ECS architecture.
///
/// Orchestrates all game systems through a centralized `World` containing entities,
/// components, and resources, while a `Schedule` defines system execution order.
/// Handles initialization of graphics resources, entity spawning, and per-frame
/// game logic coordination. SDL2 resources are stored as `NonSend` to respect
/// thread safety requirements while integrating with the ECS.
pub struct Game {
    pub world: World,
    pub schedule: Schedule,
}

impl Game {
    /// Initializes the complete game state including ECS world, graphics, and entity spawning.
    ///
    /// Performs extensive setup: creates render targets and debug textures, loads and parses
    /// the sprite atlas, renders the static map to a cached texture, builds the navigation
    /// graph from the board layout, spawns Pac-Man with directional animations, creates
    /// all four ghosts with their AI behavior, and places collectible items throughout
    /// the maze. Registers event types and configures the system execution schedule.
    ///
    /// # Arguments
    ///
    /// * `canvas` - SDL2 rendering context with static lifetime for ECS storage
    /// * `texture_creator` - SDL2 texture factory for creating render targets
    /// * `event_pump` - SDL2 event polling interface for input handling
    ///
    /// # Errors
    ///
    /// Returns `GameError` for SDL2 failures, asset loading problems, atlas parsing
    /// errors, or entity initialization issues.
    pub fn new(
        mut canvas: Canvas<Window>,
        ttf_context: sdl2::ttf::Sdl2TtfContext,
        texture_creator: TextureCreator<WindowContext>,
        mut event_pump: EventPump,
    ) -> GameResult<Game> {
        info!("Starting game initialization");

        debug!("Disabling unnecessary SDL events");
        Self::disable_sdl_events(&mut event_pump);

        debug!("Setting up textures and fonts");
        let (backbuffer, mut map_texture, debug_texture, ttf_atlas) =
            Self::setup_textures_and_fonts(&mut canvas, &texture_creator, ttf_context)?;
        trace!("Yielding after texture setup");
        platform::yield_to_browser();

        debug!("Initializing audio subsystem");
        let audio = crate::audio::Audio::new();
        trace!("Yielding after audio init");
        platform::yield_to_browser();

        debug!("Loading sprite atlas and map tiles");
        let (mut atlas, map_tiles) = Self::load_atlas_and_map_tiles(&texture_creator)?;
        trace!("Yielding after atlas load");
        platform::yield_to_browser();

        debug!("Rendering static map to texture cache");
        canvas
            .with_texture_canvas(&mut map_texture, |map_canvas| {
                MapRenderer::render_map(map_canvas, &mut atlas, &map_tiles);
            })
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        trace!("Yielding after map render");
        platform::yield_to_browser();

        debug!("Building navigation graph from map layout");
        let map = Map::new(constants::RAW_BOARD)?;

        debug!("Creating player animations and bundle");
        let (player_animation, player_start_sprite) = Self::create_player_animations(&atlas)?;
        let player_bundle = Self::create_player_bundle(&map, player_animation, player_start_sprite);

        debug!("Creating death animation sequence");
        let death_animation = Self::create_death_animation(&atlas)?;

        debug!("Initializing ECS world and system schedule");
        let mut world = World::default();
        let mut schedule = Schedule::default();

        debug!("Setting up ECS event registry and observers");
        Self::setup_ecs(&mut world);

        world.add_observer(systems::spawn_fruit_observer);

        debug!("Inserting resources into ECS world");
        Self::insert_resources(
            &mut world,
            map,
            audio,
            atlas,
            event_pump,
            canvas,
            backbuffer,
            map_texture,
            debug_texture,
            ttf_atlas,
            death_animation,
        )?;

        debug!("Configuring system execution schedule");
        Self::configure_schedule(&mut schedule);

        debug!("Spawning player entity");
        world.spawn(player_bundle).insert((Frozen, Visibility::hidden()));

        info!("Spawning game entities");
        Self::spawn_ghosts(&mut world)?;
        Self::spawn_items(&mut world)?;

        info!("Game initialization completed successfully");
        Ok(Game { world, schedule })
    }

    fn disable_sdl_events(event_pump: &mut EventPump) {
        for event_type in [
            EventType::JoyAxisMotion,
            EventType::JoyBallMotion,
            EventType::JoyHatMotion,
            EventType::JoyButtonDown,
            EventType::JoyButtonUp,
            EventType::JoyDeviceAdded,
            EventType::JoyDeviceRemoved,
            EventType::ControllerAxisMotion,
            EventType::ControllerButtonDown,
            EventType::ControllerButtonUp,
            EventType::ControllerDeviceAdded,
            EventType::ControllerDeviceRemoved,
            EventType::ControllerDeviceRemapped,
            EventType::ControllerTouchpadDown,
            EventType::ControllerTouchpadMotion,
            EventType::ControllerTouchpadUp,
            EventType::DollarGesture,
            EventType::DollarRecord,
            EventType::MultiGesture,
            EventType::ClipboardUpdate,
            EventType::DropFile,
            EventType::DropText,
            EventType::DropBegin,
            EventType::DropComplete,
            EventType::AudioDeviceAdded,
            EventType::AudioDeviceRemoved,
            EventType::RenderTargetsReset,
            EventType::RenderDeviceReset,
            EventType::LocaleChanged,
            EventType::TextInput,
            EventType::TextEditing,
            EventType::Display,
            EventType::MouseWheel,
            EventType::AppDidEnterBackground,
            EventType::AppWillEnterForeground,
            EventType::AppWillEnterBackground,
            EventType::AppDidEnterForeground,
            EventType::AppLowMemory,
            EventType::AppTerminating,
            EventType::User,
            EventType::Last,
        ] {
            event_pump.disable_event(event_type);
        }
    }

    fn setup_textures_and_fonts(
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        ttf_context: sdl2::ttf::Sdl2TtfContext,
    ) -> GameResult<(
        sdl2::render::Texture,
        sdl2::render::Texture,
        sdl2::render::Texture,
        crate::texture::ttf::TtfAtlas,
    )> {
        trace!("Creating backbuffer texture");
        let mut backbuffer = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        backbuffer.set_scale_mode(ScaleMode::Nearest);
        platform::yield_to_browser();

        trace!("Creating map texture");
        let mut map_texture = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        map_texture.set_scale_mode(ScaleMode::Nearest);
        platform::yield_to_browser();

        trace!("Creating debug texture");
        let output_size = constants::LARGE_CANVAS_SIZE;
        let mut debug_texture = texture_creator
            .create_texture_target(Some(sdl2::pixels::PixelFormatEnum::ARGB8888), output_size.x, output_size.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        debug_texture.set_blend_mode(BlendMode::Blend);
        debug_texture.set_scale_mode(ScaleMode::Nearest);
        platform::yield_to_browser();

        trace!("Loading font");
        let font_data: &'static [u8] = Asset::Font.get_bytes()?.to_vec().leak();
        let font_asset = RWops::from_bytes(font_data).map_err(|_| GameError::Sdl("Failed to load font".to_string()))?;
        let debug_font = ttf_context
            .load_font_from_rwops(font_asset, constants::ui::DEBUG_FONT_SIZE)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        trace!("Creating TTF atlas");
        let mut ttf_atlas = crate::texture::ttf::TtfAtlas::new(texture_creator, &debug_font)?;
        platform::yield_to_browser();

        trace!("Populating TTF atlas");
        ttf_atlas.populate_atlas(canvas, texture_creator, &debug_font)?;

        Ok((backbuffer, map_texture, debug_texture, ttf_atlas))
    }

    fn load_atlas_and_map_tiles(texture_creator: &TextureCreator<WindowContext>) -> GameResult<(SpriteAtlas, Vec<AtlasTile>)> {
        trace!("Loading atlas image from embedded assets");
        let atlas_bytes = Asset::AtlasImage.get_bytes()?;
        let atlas_texture = texture_creator.load_texture_bytes(&atlas_bytes).map_err(|e| {
            if e.to_string().contains("format") || e.to_string().contains("unsupported") {
                GameError::Texture(crate::error::TextureError::InvalidFormat(format!(
                    "Unsupported texture format: {e}"
                )))
            } else {
                GameError::Texture(crate::error::TextureError::LoadFailed(e.to_string()))
            }
        })?;

        debug!(frame_count = ATLAS_FRAMES.len(), "Creating sprite atlas from texture");
        let atlas_mapper = AtlasMapper {
            frames: ATLAS_FRAMES.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        };
        let atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

        trace!("Extracting map tile sprites from atlas");
        let mut map_tiles = Vec::with_capacity(35);
        for i in 0..35 {
            let tile_name = GameSprite::Maze(MazeSprite::Tile(i)).to_path();
            let tile = atlas.get_tile(&tile_name)?;
            map_tiles.push(tile);
        }

        Ok((atlas, map_tiles))
    }

    fn create_player_animations(atlas: &SpriteAtlas) -> GameResult<(DirectionalAnimation, AtlasTile)> {
        let up_moving_tiles = [
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 0)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 1)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
        ];
        let down_moving_tiles = [
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 0)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 1)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
        ];
        let left_moving_tiles = [
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 0)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path())?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
        ];
        let right_moving_tiles = [
            SpriteAtlas::get_tile(
                atlas,
                &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 0)).to_path(),
            )?,
            SpriteAtlas::get_tile(
                atlas,
                &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 1)).to_path(),
            )?,
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?,
        ];

        let moving_tiles = DirectionalTiles::new(
            TileSequence::new(&up_moving_tiles),
            TileSequence::new(&down_moving_tiles),
            TileSequence::new(&left_moving_tiles),
            TileSequence::new(&right_moving_tiles),
        );

        let up_stopped_tile =
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Up, 1)).to_path())?;
        let down_stopped_tile =
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Down, 1)).to_path())?;
        let left_stopped_tile =
            SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path())?;
        let right_stopped_tile = SpriteAtlas::get_tile(
            atlas,
            &GameSprite::Pacman(PacmanSprite::Moving(Direction::Right, 1)).to_path(),
        )?;

        let stopped_tiles = DirectionalTiles::new(
            TileSequence::new(&[up_stopped_tile]),
            TileSequence::new(&[down_stopped_tile]),
            TileSequence::new(&[left_stopped_tile]),
            TileSequence::new(&[right_stopped_tile]),
        );

        let player_animation = DirectionalAnimation::new(moving_tiles, stopped_tiles, 5);
        let player_start_sprite = SpriteAtlas::get_tile(atlas, &GameSprite::Pacman(PacmanSprite::Full).to_path())?;

        Ok((player_animation, player_start_sprite))
    }

    fn create_death_animation(atlas: &SpriteAtlas) -> GameResult<LinearAnimation> {
        let mut death_tiles = Vec::new();
        for i in 0..=10 {
            // Assuming death animation has 11 frames named pacman/die_0, pacman/die_1, etc.
            let tile = atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Dying(i)).to_path())?;
            death_tiles.push(tile);
        }

        let tile_sequence = TileSequence::new(&death_tiles);
        Ok(LinearAnimation::new(tile_sequence, 8)) // 8 ticks per frame, non-looping
    }

    fn create_player_bundle(map: &Map, player_animation: DirectionalAnimation, player_start_sprite: AtlasTile) -> PlayerBundle {
        PlayerBundle {
            player: PlayerControlled,
            position: Position::Stopped {
                node: map.start_positions.pacman,
            },
            velocity: Velocity {
                speed: constants::mechanics::PLAYER_SPEED,
                direction: Direction::Left,
            },
            movement_modifiers: MovementModifiers::default(),
            buffered_direction: BufferedDirection::None,
            sprite: Renderable {
                sprite: player_start_sprite,
                layer: 0,
            },
            directional_animation: player_animation,
            entity_type: EntityType::Player,
            collider: Collider {
                size: constants::collider::PLAYER_SIZE,
            },
            pacman_collider: PacmanCollider,
        }
    }

    fn setup_ecs(world: &mut World) {
        EventRegistry::register_event::<GameError>(world);
        EventRegistry::register_event::<GameEvent>(world);
        EventRegistry::register_event::<AudioEvent>(world);
        EventRegistry::register_event::<StageTransition>(world);
        EventRegistry::register_event::<CollisionTrigger>(world);

        world.add_observer(
            |event: Trigger<GameEvent>, mut state: ResMut<GlobalState>, _score: ResMut<ScoreResource>| {
                if matches!(*event, GameEvent::Command(GameCommand::Exit)) {
                    state.exit = true;
                }
            },
        );

        world.add_observer(ghost_collision_observer);
        world.add_observer(item_collision_observer);
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_resources(
        world: &mut World,
        map: Map,
        audio: crate::audio::Audio,
        atlas: SpriteAtlas,
        event_pump: EventPump,
        canvas: Canvas<Window>,
        backbuffer: sdl2::render::Texture,
        map_texture: sdl2::render::Texture,
        debug_texture: sdl2::render::Texture,
        ttf_atlas: crate::texture::ttf::TtfAtlas,
        death_animation: LinearAnimation,
    ) -> GameResult<()> {
        world.insert_non_send_resource(atlas);
        world.insert_resource(Self::create_ghost_animations(world.non_send_resource::<SpriteAtlas>())?);
        let player_animation = Self::create_player_animations(world.non_send_resource::<SpriteAtlas>())?.0;
        world.insert_resource(PlayerAnimation(player_animation));
        world.insert_resource(PlayerDeathAnimation(death_animation));

        world.insert_resource(FruitSprites::default());
        world.insert_resource(BatchedLinesResource::new(&map, constants::LARGE_SCALE));
        world.insert_resource(map);
        world.insert_resource(GlobalState { exit: false });
        world.insert_resource(PlayerLives::default());
        world.insert_resource(ScoreResource(0));
        world.insert_resource(PelletCount(0));
        world.insert_resource(SystemTimings::default());
        world.insert_resource(Timing::default());
        world.insert_resource(Bindings::default());
        world.insert_resource(DeltaTime { seconds: 0.0, ticks: 0 });
        world.insert_resource(RenderDirty::default());
        world.insert_resource(DebugState::default());
        world.insert_resource(AudioState::default());
        world.insert_resource(IntroPlayed::default());
        world.insert_resource(CursorPosition::default());
        world.insert_resource(TouchState::default());
        // On Emscripten, start in WaitingForInteraction state due to browser autoplay policy.
        // The game will transition to Starting when the user clicks or presses a key.
        #[cfg(target_os = "emscripten")]
        world.insert_resource(GameStage::WaitingForInteraction);

        #[cfg(not(target_os = "emscripten"))]
        world.insert_resource(GameStage::Starting(StartupSequence::TextOnly {
            remaining_ticks: constants::startup::STARTUP_FRAMES,
        }));
        world.insert_resource(PauseState::default());

        world.insert_non_send_resource(event_pump);
        world.insert_non_send_resource::<&mut Canvas<Window>>(Box::leak(Box::new(canvas)));
        world.insert_non_send_resource(BackbufferResource(backbuffer));
        world.insert_non_send_resource(MapTextureResource(map_texture));
        world.insert_non_send_resource(DebugTextureResource(debug_texture));
        world.insert_non_send_resource(TtfAtlasResource(ttf_atlas));
        world.insert_non_send_resource(AudioResource(audio));
        Ok(())
    }

    fn configure_schedule(schedule: &mut Schedule) {
        let stage_system = profile(SystemId::Stage, systems::stage_system);
        let input_system = profile(SystemId::Input, systems::input::input_system);
        let pause_system = profile(SystemId::Input, systems::handle_pause_command);
        let player_control_system = profile(SystemId::PlayerControls, systems::player_control_system);
        let player_movement_system = profile(SystemId::PlayerMovement, systems::player_movement_system);
        let player_tunnel_slowdown_system = profile(SystemId::PlayerMovement, systems::player::player_tunnel_slowdown_system);
        let ghost_movement_system = profile(SystemId::Ghost, ghost_movement_system);
        let collision_system = profile(SystemId::Collision, collision_system);
        let audio_system = profile(SystemId::Audio, audio_system);
        let blinking_system = profile(SystemId::Blinking, blinking_system);
        let directional_render_system = profile(SystemId::DirectionalRender, directional_render_system);
        let linear_render_system = profile(SystemId::LinearRender, linear_render_system);
        let dirty_render_system = profile(SystemId::DirtyRender, dirty_render_system);
        let hud_render_system = profile(SystemId::HudRender, hud_render_system);
        let player_life_sprite_system = profile(SystemId::HudRender, player_life_sprite_system);
        let fruit_sprite_system = profile(SystemId::HudRender, fruit_sprite_system);
        let present_system = profile(SystemId::Present, present_system);
        let unified_ghost_state_system = profile(SystemId::GhostStateAnimation, ghost_state_system);
        let eaten_ghost_system = profile(SystemId::EatenGhost, eaten_ghost_system);
        let time_to_live_system = profile(SystemId::TimeToLive, time_to_live_system);
        let manage_pause_state_system = profile(SystemId::PauseManager, systems::state::manage_pause_state_system);

        // Input system should always run to prevent SDL event pump from blocking
        let input_systems = (
            input_system.run_if(|mut local: Local<u8>| {
                *local = local.wrapping_add(1u8);
                // run every nth frame
                *local % 2 == 0
            }),
            player_control_system,
            pause_system,
            #[cfg(not(target_os = "emscripten"))]
            profile(SystemId::Input, systems::handle_fullscreen_command),
        )
            .chain();

        // .run_if(|game_state: Res<GameStage>| matches!(*game_state, GameStage::Playing));

        schedule
            .add_systems((
                input_systems.in_set(GameplaySet::Input),
                time_to_live_system.before(GameplaySet::Update),
                (
                    player_movement_system,
                    player_tunnel_slowdown_system,
                    ghost_movement_system,
                    eaten_ghost_system,
                    collision_system,
                    unified_ghost_state_system,
                )
                    .in_set(GameplaySet::Update),
                (
                    blinking_system,
                    directional_render_system,
                    linear_render_system,
                    player_life_sprite_system,
                    fruit_sprite_system,
                )
                    .in_set(RenderSet::Animation),
                stage_system.in_set(GameplaySet::Respond),
                (
                    (|mut dirty: ResMut<RenderDirty>, score: Res<ScoreResource>, stage: Res<GameStage>| {
                        dirty.0 |= score.is_changed() || stage.is_changed();
                    }),
                    dirty_render_system.run_if(|dirty: Res<RenderDirty>| dirty.0.not()),
                    combined_render_system,
                    hud_render_system,
                    touch_ui_render_system,
                )
                    .chain()
                    .in_set(RenderSet::Draw),
                (present_system, audio_system).chain().in_set(RenderSet::Present),
                manage_pause_state_system.after(GameplaySet::Update),
            ))
            .configure_sets((
                GameplaySet::Input,
                GameplaySet::Update.run_if(|paused: Res<PauseState>| !paused.active()),
                GameplaySet::Respond.run_if(|paused: Res<PauseState>| !paused.active()),
                RenderSet::Animation.run_if(|paused: Res<PauseState>| !paused.active()),
                RenderSet::Draw,
                RenderSet::Present,
            ));
    }

    fn spawn_items(world: &mut World) -> GameResult<()> {
        trace!("Loading item sprites from atlas");
        let pellet_sprite = SpriteAtlas::get_tile(
            world.non_send_resource::<SpriteAtlas>(),
            &GameSprite::Maze(MazeSprite::Pellet).to_path(),
        )?;
        let energizer_sprite = SpriteAtlas::get_tile(
            world.non_send_resource::<SpriteAtlas>(),
            &GameSprite::Maze(MazeSprite::Energizer).to_path(),
        )?;

        let nodes: Vec<(NodeId, EntityType, AtlasTile, f32)> = world
            .resource::<Map>()
            .iter_nodes()
            .filter_map(|(id, tile)| match tile {
                MapTile::Pellet => Some((*id, EntityType::Pellet, pellet_sprite, constants::collider::PELLET_SIZE)),
                MapTile::PowerPellet => Some((
                    *id,
                    EntityType::PowerPellet,
                    energizer_sprite,
                    constants::collider::POWER_PELLET_SIZE,
                )),
                _ => None,
            })
            .collect();

        info!(
            pellet_count = nodes.iter().filter(|(_, t, _, _)| *t == EntityType::Pellet).count(),
            power_pellet_count = nodes.iter().filter(|(_, t, _, _)| *t == EntityType::PowerPellet).count(),
            "Spawning collectible items"
        );

        for (id, item_type, sprite, size) in nodes {
            let mut item = world.spawn(ItemBundle {
                position: Position::Stopped { node: id },
                sprite: Renderable { sprite, layer: 1 },
                entity_type: item_type,
                collider: Collider { size },
                item_collider: ItemCollider,
            });

            if item_type == EntityType::PowerPellet {
                item.insert((Frozen, Blinking::new(constants::ui::POWER_PELLET_BLINK_RATE)));
            }
        }
        Ok(())
    }

    /// Creates and spawns all four ghosts with unique AI personalities and directional animations.
    ///
    /// # Errors
    ///
    /// Returns `GameError::Texture` if any ghost sprite cannot be found in the atlas,
    /// typically indicating missing or misnamed sprite files.
    fn spawn_ghosts(world: &mut World) -> GameResult<()> {
        trace!("Spawning ghost entities with AI personalities");
        // Extract the data we need first to avoid borrow conflicts
        let ghost_start_positions = {
            let map = world.resource::<Map>();
            [
                (Ghost::Blinky, map.start_positions.blinky),
                (Ghost::Pinky, map.start_positions.pinky),
                (Ghost::Inky, map.start_positions.inky),
                (Ghost::Clyde, map.start_positions.clyde),
            ]
        };

        for (ghost_type, start_node) in ghost_start_positions {
            // Create the ghost bundle in a separate scope to manage borrows
            let ghost = {
                let animations = world.resource::<GhostAnimations>().get_normal(&ghost_type).unwrap().clone();
                let atlas = world.non_send_resource::<SpriteAtlas>();
                let sprite_path = GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 0)).to_path();

                GhostBundle {
                    ghost: ghost_type,
                    position: Position::Stopped { node: start_node },
                    velocity: Velocity {
                        speed: ghost_type.base_speed(),
                        direction: Direction::Left,
                    },
                    sprite: Renderable {
                        sprite: SpriteAtlas::get_tile(atlas, &sprite_path)?,
                        layer: 0,
                    },
                    directional_animation: animations,
                    entity_type: EntityType::Ghost,
                    collider: Collider {
                        size: constants::collider::GHOST_SIZE,
                    },
                    ghost_collider: GhostCollider,
                    ghost_state: GhostState::Normal,
                    last_animation_state: LastAnimationState(GhostAnimation::Normal),
                }
            };

            let entity = world.spawn(ghost).insert((Frozen, Visibility::hidden())).id();
            trace!(ghost = ?ghost_type, entity = ?entity, start_node, "Spawned ghost entity");
        }

        info!("All ghost entities spawned successfully");
        Ok(())
    }

    fn create_ghost_animations(atlas: &SpriteAtlas) -> GameResult<GhostAnimations> {
        // Eaten (eyes) animations - single tile per direction
        let up_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Up)).to_path())?;
        let down_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Down)).to_path())?;
        let left_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Left)).to_path())?;
        let right_eye = atlas.get_tile(&GameSprite::Ghost(GhostSprite::Eyes(Direction::Right)).to_path())?;

        let eyes_tiles = DirectionalTiles::new(
            TileSequence::new(&[up_eye]),
            TileSequence::new(&[down_eye]),
            TileSequence::new(&[left_eye]),
            TileSequence::new(&[right_eye]),
        );
        let eyes = DirectionalAnimation::new(eyes_tiles.clone(), eyes_tiles, animation::GHOST_EATEN_SPEED);

        let mut animations = HashMap::new();

        for ghost_type in [Ghost::Blinky, Ghost::Pinky, Ghost::Inky, Ghost::Clyde] {
            // Normal animations - create directional tiles for each direction
            let up_tiles = [
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Up, 0)).to_path())?,
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Up, 1)).to_path())?,
            ];
            let down_tiles = [
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Down, 0)).to_path())?,
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Down, 1)).to_path())?,
            ];
            let left_tiles = [
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 0)).to_path())?,
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 1)).to_path())?,
            ];
            let right_tiles = [
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Right, 0)).to_path())?,
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Right, 1)).to_path())?,
            ];

            let normal_moving = DirectionalTiles::new(
                TileSequence::new(&up_tiles),
                TileSequence::new(&down_tiles),
                TileSequence::new(&left_tiles),
                TileSequence::new(&right_tiles),
            );
            let normal = DirectionalAnimation::new(normal_moving.clone(), normal_moving, animation::GHOST_NORMAL_SPEED);

            animations.insert(ghost_type, normal);
        }

        let (frightened, frightened_flashing) = {
            // Load frightened animation tiles (same for all ghosts)
            let frightened_blue_a =
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::Blue, 0)).to_path())?;
            let frightened_blue_b =
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::Blue, 1)).to_path())?;
            let frightened_white_a =
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::White, 0)).to_path())?;
            let frightened_white_b =
                atlas.get_tile(&GameSprite::Ghost(GhostSprite::Frightened(FrightenedColor::White, 1)).to_path())?;

            (
                LinearAnimation::new(
                    TileSequence::new(&[frightened_blue_a, frightened_blue_b]),
                    animation::GHOST_NORMAL_SPEED,
                ),
                LinearAnimation::new(
                    TileSequence::new(&[frightened_blue_a, frightened_white_a, frightened_blue_b, frightened_white_b]),
                    animation::GHOST_FRIGHTENED_SPEED,
                ),
            )
        };

        Ok(GhostAnimations::new(animations, eyes, frightened, frightened_flashing))
    }

    /// Starts the game after user interaction (Emscripten only).
    ///
    /// Transitions from WaitingForInteraction to Starting state and unlocks audio.
    /// Called from JavaScript when the user clicks or presses a key.
    #[cfg(target_os = "emscripten")]
    pub fn start(&mut self) {
        use crate::systems::state::{GameStage, StartupSequence};

        // Unlock audio now that user has interacted
        if let Some(mut audio) = self.world.get_non_send_resource_mut::<AudioResource>() {
            audio.0.unlock();
        }

        // Transition to Starting state if we're waiting
        if let Some(mut stage) = self.world.get_resource_mut::<GameStage>() {
            if matches!(*stage, GameStage::WaitingForInteraction) {
                tracing::info!("User interaction detected, starting game");
                *stage = GameStage::Starting(StartupSequence::TextOnly {
                    remaining_ticks: constants::startup::STARTUP_FRAMES,
                });
            }
        }
    }

    /// Executes one frame of game logic by running all scheduled ECS systems.
    ///
    /// Updates the world's delta time resource and runs the complete system pipeline:
    /// input processing, entity movement, collision detection, item collection,
    /// audio playback, animation updates, and rendering. Each system operates on
    /// relevant entities and modifies world state, with the schedule ensuring
    /// proper execution order and data dependencies.
    ///
    /// # Arguments
    ///
    /// * `dt` - Frame delta time in seconds for time-based animations and movement
    ///
    /// # Returns
    ///
    /// `true` if the game should terminate (exit command received), `false` to continue
    pub fn tick(&mut self, dt: f32) -> bool {
        self.world.insert_resource(DeltaTime { seconds: dt, ticks: 1 });

        // Note: We don't need to read the current tick here since we increment it after running systems

        // Measure total frame time including all systems
        let start = std::time::Instant::now();
        self.schedule.run(&mut self.world);
        let total_duration = start.elapsed();

        // Increment tick counter and record the total timing
        if let (Some(timings), Some(timing)) = (
            self.world.get_resource::<systems::profiling::SystemTimings>(),
            self.world.get_resource::<Timing>(),
        ) {
            let new_tick = timing.increment_tick();
            timings.add_total_timing(total_duration, new_tick);

            // Calculate dynamic threshold based on actual frame budget
            // Use dt to determine expected frame time, with 80% as threshold to account for normal variance
            // Desktop uses LOOP_TIME (~16.67ms), WebAssembly adapts to requestAnimationFrame timing
            let frame_budget_ms = (dt * 1000.0 * 1.2) as u128;

            // Log performance warnings for slow frames
            if total_duration.as_millis() > frame_budget_ms {
                let slowest_systems = timings.get_slowest_systems();
                let systems_context = if slowest_systems.is_empty() {
                    "No specific systems identified".to_string()
                } else {
                    slowest_systems
                        .iter()
                        .map(|(id, duration)| format!("{} ({:.2?})", id, duration))
                        .collect::<Vec<String>>()
                        .join(", ")
                };

                warn!(
                    total = format!("{:.3?}", total_duration),
                    tick = new_tick,
                    systems = systems_context,
                    budget = format!("{:.1}ms", frame_budget_ms),
                    "Frame took longer than expected"
                );
            }
        }

        let state = self
            .world
            .get_resource::<GlobalState>()
            .expect("GlobalState could not be acquired");

        state.exit
    }
}
