//! This module contains the main game logic and state.

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

use std::collections::HashMap;

use crate::constants::{self, animation, MapTile, CANVAS_SIZE};
use crate::error::{GameError, GameResult, TextureError};
use crate::events::GameEvent;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::blinking::Blinking;
use crate::systems::components::{GhostAnimation, GhostState, LastAnimationState};
use crate::systems::movement::{BufferedDirection, Position, Velocity};
use crate::systems::profiling::SystemId;
use crate::systems::render::RenderDirty;
use crate::systems::{
    self, combined_render_system, ghost_collision_system, present_system, Hidden, LinearAnimation, MovementModifiers, NodeId,
};
use crate::systems::{
    audio_system, blinking_system, collision_system, directional_render_system, dirty_render_system, eaten_ghost_system,
    ghost_movement_system, ghost_state_system, hud_render_system, item_system, linear_render_system, profile, AudioEvent,
    AudioResource, AudioState, BackbufferResource, Collider, DebugState, DebugTextureResource, DeltaTime, DirectionalAnimation,
    EntityType, Frozen, Ghost, GhostAnimations, GhostBundle, GhostCollider, GlobalState, ItemBundle, ItemCollider,
    MapTextureResource, PacmanCollider, PlayerBundle, PlayerControlled, Renderable, ScoreResource, StartupSequence,
    SystemTimings,
};
use crate::texture::animated::{DirectionalTiles, TileSequence};
use crate::texture::sprite::AtlasTile;
use bevy_ecs::event::EventRegistry;
use bevy_ecs::observer::Trigger;
use bevy_ecs::schedule::common_conditions::resource_changed;
use bevy_ecs::schedule::{Condition, IntoScheduleConfigs, Schedule, SystemSet};
use bevy_ecs::system::{Local, ResMut};
use bevy_ecs::world::World;
use glam::UVec2;
use sdl2::event::EventType;
use sdl2::image::LoadTexture;
use sdl2::render::{BlendMode, Canvas, ScaleMode, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::{
    asset::{get_asset_bytes, Asset},
    events::GameCommand,
    map::render::MapRenderer,
    systems::debug::{BatchedLinesResource, TtfAtlasResource},
    systems::input::{Bindings, CursorPosition},
    texture::sprite::{AtlasMapper, SpriteAtlas},
};

/// System set for all rendering systems to ensure they run after gameplay logic
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RenderSet;

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
        texture_creator: TextureCreator<WindowContext>,
        mut event_pump: EventPump,
    ) -> GameResult<Game> {
        // Disable uninteresting events
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
            EventType::FingerDown,
            EventType::FingerUp,
            EventType::FingerMotion,
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
            EventType::Window,
            EventType::MouseWheel,
            // EventType::MouseMotion,
            EventType::MouseButtonDown,
            EventType::MouseButtonUp,
            EventType::MouseButtonDown,
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

        let ttf_context = Box::leak(Box::new(sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?));
        let mut backbuffer = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        backbuffer.set_scale_mode(ScaleMode::Nearest);

        let mut map_texture = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        map_texture.set_scale_mode(ScaleMode::Nearest);

        // Create debug texture at output resolution for crisp debug rendering
        let output_size = canvas.output_size().unwrap();
        let mut debug_texture = texture_creator
            .create_texture_target(Some(sdl2::pixels::PixelFormatEnum::ARGB8888), output_size.0, output_size.1)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        // Debug texture is copied over the backbuffer, it requires transparency abilities
        debug_texture.set_blend_mode(BlendMode::Blend);
        debug_texture.set_scale_mode(ScaleMode::Nearest);

        // Create debug text atlas for efficient debug rendering
        let font_data: &'static [u8] = get_asset_bytes(Asset::Font)?.to_vec().leak();
        let font_asset = RWops::from_bytes(font_data).map_err(|_| GameError::Sdl("Failed to load font".to_string()))?;
        let debug_font = ttf_context
            .load_font_from_rwops(font_asset, constants::ui::DEBUG_FONT_SIZE)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        let mut ttf_atlas = crate::texture::ttf::TtfAtlas::new(&texture_creator, &debug_font)?;
        // Populate the atlas with actual character data
        ttf_atlas.populate_atlas(&mut canvas, &texture_creator, &debug_font)?;

        // Initialize audio system
        let audio = crate::audio::Audio::new();

        // Load atlas and create map texture
        let atlas_bytes = get_asset_bytes(Asset::AtlasImage)?;
        let atlas_texture = texture_creator.load_texture_bytes(&atlas_bytes).map_err(|e| {
            if e.to_string().contains("format") || e.to_string().contains("unsupported") {
                GameError::Texture(crate::error::TextureError::InvalidFormat(format!(
                    "Unsupported texture format: {e}"
                )))
            } else {
                GameError::Texture(crate::error::TextureError::LoadFailed(e.to_string()))
            }
        })?;

        let atlas_mapper = AtlasMapper {
            frames: ATLAS_FRAMES.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        };
        let mut atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

        // Create map tiles
        let mut map_tiles = Vec::with_capacity(35);
        for i in 0..35 {
            let tile_name = format!("maze/tiles/{}.png", i);
            let tile = atlas.get_tile(&tile_name).unwrap();
            map_tiles.push(tile);
        }

        // Render map to texture
        canvas
            .with_texture_canvas(&mut map_texture, |map_canvas| {
                MapRenderer::render_map(map_canvas, &mut atlas, &map_tiles);
            })
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        let map = Map::new(constants::RAW_BOARD)?;

        // Create directional animated textures for Pac-Man
        let up_moving_tiles = [
            SpriteAtlas::get_tile(&atlas, "pacman/up_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/up_a.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/up_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/up_b.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
        ];
        let down_moving_tiles = [
            SpriteAtlas::get_tile(&atlas, "pacman/down_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/down_a.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/down_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/down_b.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
        ];
        let left_moving_tiles = [
            SpriteAtlas::get_tile(&atlas, "pacman/left_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/left_a.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/left_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/left_b.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
        ];
        let right_moving_tiles = [
            SpriteAtlas::get_tile(&atlas, "pacman/right_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/right_a.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/right_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/right_b.png".to_string())))?,
            SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
        ];

        let moving_tiles = DirectionalTiles::new(
            TileSequence::new(&up_moving_tiles),
            TileSequence::new(&down_moving_tiles),
            TileSequence::new(&left_moving_tiles),
            TileSequence::new(&right_moving_tiles),
        );

        let up_stopped_tile = SpriteAtlas::get_tile(&atlas, "pacman/up_b.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/up_b.png".to_string())))?;
        let down_stopped_tile = SpriteAtlas::get_tile(&atlas, "pacman/down_b.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/down_b.png".to_string())))?;
        let left_stopped_tile = SpriteAtlas::get_tile(&atlas, "pacman/left_b.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/left_b.png".to_string())))?;
        let right_stopped_tile = SpriteAtlas::get_tile(&atlas, "pacman/right_b.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/right_b.png".to_string())))?;

        let stopped_tiles = DirectionalTiles::new(
            TileSequence::new(&[up_stopped_tile]),
            TileSequence::new(&[down_stopped_tile]),
            TileSequence::new(&[left_stopped_tile]),
            TileSequence::new(&[right_stopped_tile]),
        );

        let player = PlayerBundle {
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
                sprite: SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
                layer: 0,
            },
            directional_animation: DirectionalAnimation::new(moving_tiles, stopped_tiles, 5),
            entity_type: EntityType::Player,
            collider: Collider {
                size: constants::collider::PLAYER_GHOST_SIZE,
            },
            pacman_collider: PacmanCollider,
        };

        let mut world = World::default();
        let mut schedule = Schedule::default();

        EventRegistry::register_event::<GameError>(&mut world);
        EventRegistry::register_event::<GameEvent>(&mut world);
        EventRegistry::register_event::<AudioEvent>(&mut world);

        let scale =
            (UVec2::from(canvas.output_size().unwrap()).as_vec2() / UVec2::from(canvas.logical_size()).as_vec2()).min_element();

        world.insert_resource(BatchedLinesResource::new(&map, scale));
        world.insert_resource(Self::create_ghost_animations(&atlas)?);
        world.insert_resource(map);
        world.insert_resource(GlobalState { exit: false });
        world.insert_resource(ScoreResource(0));
        world.insert_resource(SystemTimings::default());
        world.insert_resource(Bindings::default());
        world.insert_resource(DeltaTime(0f32));
        world.insert_resource(RenderDirty::default());
        world.insert_resource(DebugState::default());
        world.insert_resource(AudioState::default());
        world.insert_resource(CursorPosition::default());
        world.insert_resource(StartupSequence::new(
            constants::startup::STARTUP_FRAMES,
            constants::startup::STARTUP_TICKS_PER_FRAME,
        ));

        world.insert_non_send_resource(atlas);
        world.insert_non_send_resource(event_pump);
        world.insert_non_send_resource::<&mut Canvas<Window>>(Box::leak(Box::new(canvas)));
        world.insert_non_send_resource(BackbufferResource(backbuffer));
        world.insert_non_send_resource(MapTextureResource(map_texture));
        world.insert_non_send_resource(DebugTextureResource(debug_texture));
        world.insert_non_send_resource(TtfAtlasResource(ttf_atlas));
        world.insert_non_send_resource(AudioResource(audio));

        world.add_observer(
            |event: Trigger<GameEvent>, mut state: ResMut<GlobalState>, _score: ResMut<ScoreResource>| {
                if matches!(*event, GameEvent::Command(GameCommand::Exit)) {
                    state.exit = true;
                }
            },
        );

        let input_system = profile(SystemId::Input, systems::input::input_system);
        let player_control_system = profile(SystemId::PlayerControls, systems::player_control_system);
        let player_movement_system = profile(SystemId::PlayerMovement, systems::player_movement_system);
        let startup_stage_system = profile(SystemId::Stage, systems::startup_stage_system);
        let player_tunnel_slowdown_system = profile(SystemId::PlayerMovement, systems::player::player_tunnel_slowdown_system);
        let ghost_movement_system = profile(SystemId::Ghost, ghost_movement_system);
        let collision_system = profile(SystemId::Collision, collision_system);
        let ghost_collision_system = profile(SystemId::GhostCollision, ghost_collision_system);

        let item_system = profile(SystemId::Item, item_system);
        let audio_system = profile(SystemId::Audio, audio_system);
        let blinking_system = profile(SystemId::Blinking, blinking_system);
        let directional_render_system = profile(SystemId::DirectionalRender, directional_render_system);
        let linear_render_system = profile(SystemId::LinearRender, linear_render_system);
        let dirty_render_system = profile(SystemId::DirtyRender, dirty_render_system);
        let hud_render_system = profile(SystemId::HudRender, hud_render_system);
        let present_system = profile(SystemId::Present, present_system);
        let unified_ghost_state_system = profile(SystemId::GhostStateAnimation, ghost_state_system);

        let forced_dirty_system = |mut dirty: ResMut<RenderDirty>| {
            dirty.0 = true;
        };

        schedule.add_systems((
            forced_dirty_system.run_if(resource_changed::<ScoreResource>.or(resource_changed::<StartupSequence>)),
            (
                input_system.run_if(|mut local: Local<u8>| {
                    *local = local.wrapping_add(1u8);
                    // run every nth frame
                    *local % 2 == 0
                }),
                player_control_system,
                player_movement_system,
                startup_stage_system,
            )
                .chain(),
            player_tunnel_slowdown_system,
            ghost_movement_system,
            profile(SystemId::EatenGhost, eaten_ghost_system),
            unified_ghost_state_system,
            (collision_system, ghost_collision_system, item_system).chain(),
            audio_system,
            blinking_system,
            (
                directional_render_system,
                linear_render_system,
                dirty_render_system,
                combined_render_system,
                hud_render_system,
                present_system,
            )
                .chain(),
        ));

        // Spawn player and attach initial state bundle
        world.spawn(player).insert((Frozen, Hidden));

        // Spawn ghosts
        Self::spawn_ghosts(&mut world)?;

        let pellet_sprite = SpriteAtlas::get_tile(world.non_send_resource::<SpriteAtlas>(), "maze/pellet.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("maze/pellet.png".to_string())))?;
        let energizer_sprite = SpriteAtlas::get_tile(world.non_send_resource::<SpriteAtlas>(), "maze/energizer.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("maze/energizer.png".to_string())))?;

        // Build a list of item entities to spawn from the map
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

        // Construct and spawn the item entities
        for (id, item_type, sprite, size) in nodes {
            let mut item = world.spawn(ItemBundle {
                position: Position::Stopped { node: id },
                sprite: Renderable { sprite, layer: 1 },
                entity_type: item_type,
                collider: Collider { size },
                item_collider: ItemCollider,
            });

            // Make power pellets blink
            if item_type == EntityType::PowerPellet {
                item.insert((Frozen, Blinking::new(constants::ui::POWER_PELLET_BLINK_RATE)));
            }
        }

        Ok(Game { world, schedule })
    }

    /// Creates and spawns all four ghosts with unique AI personalities and directional animations.
    ///
    /// # Errors
    ///
    /// Returns `GameError::Texture` if any ghost sprite cannot be found in the atlas,
    /// typically indicating missing or misnamed sprite files.
    fn spawn_ghosts(world: &mut World) -> GameResult<()> {
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
                let animations = *world.resource::<GhostAnimations>().get_normal(&ghost_type).unwrap();
                let atlas = world.non_send_resource::<SpriteAtlas>();

                GhostBundle {
                    ghost: ghost_type,
                    position: Position::Stopped { node: start_node },
                    velocity: Velocity {
                        speed: ghost_type.base_speed(),
                        direction: Direction::Left,
                    },
                    sprite: Renderable {
                        sprite: SpriteAtlas::get_tile(atlas, &format!("ghost/{}/left_a.png", ghost_type.as_str())).ok_or_else(
                            || {
                                GameError::Texture(TextureError::AtlasTileNotFound(format!(
                                    "ghost/{}/left_a.png",
                                    ghost_type.as_str()
                                )))
                            },
                        )?,
                        layer: 0,
                    },
                    directional_animation: animations,
                    entity_type: EntityType::Ghost,
                    collider: Collider {
                        size: constants::collider::PLAYER_GHOST_SIZE,
                    },
                    ghost_collider: GhostCollider,
                    ghost_state: GhostState::Normal,
                    last_animation_state: LastAnimationState(GhostAnimation::Normal),
                }
            };

            world.spawn(ghost).insert((Frozen, Hidden));
        }

        Ok(())
    }

    fn create_ghost_animations(atlas: &SpriteAtlas) -> GameResult<GhostAnimations> {
        // Eaten (eyes) animations - single tile per direction
        let up_eye = atlas
            .get_tile("ghost/eyes/up.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/eyes/up.png".to_string())))?;
        let down_eye = atlas
            .get_tile("ghost/eyes/down.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/eyes/down.png".to_string())))?;
        let left_eye = atlas
            .get_tile("ghost/eyes/left.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/eyes/left.png".to_string())))?;
        let right_eye = atlas
            .get_tile("ghost/eyes/right.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/eyes/right.png".to_string())))?;

        let eyes_tiles = DirectionalTiles::new(
            TileSequence::new(&[up_eye]),
            TileSequence::new(&[down_eye]),
            TileSequence::new(&[left_eye]),
            TileSequence::new(&[right_eye]),
        );
        let eyes = DirectionalAnimation::new(eyes_tiles, eyes_tiles, animation::GHOST_EATEN_SPEED);

        let mut animations = HashMap::new();

        for ghost_type in [Ghost::Blinky, Ghost::Pinky, Ghost::Inky, Ghost::Clyde] {
            // Normal animations - create directional tiles for each direction
            let up_tiles = [
                atlas
                    .get_tile(&format!("ghost/{}/up_a.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/up_a.png",
                            ghost_type.as_str()
                        )))
                    })?,
                atlas
                    .get_tile(&format!("ghost/{}/up_b.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/up_b.png",
                            ghost_type.as_str()
                        )))
                    })?,
            ];
            let down_tiles = [
                atlas
                    .get_tile(&format!("ghost/{}/down_a.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/down_a.png",
                            ghost_type.as_str()
                        )))
                    })?,
                atlas
                    .get_tile(&format!("ghost/{}/down_b.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/down_b.png",
                            ghost_type.as_str()
                        )))
                    })?,
            ];
            let left_tiles = [
                atlas
                    .get_tile(&format!("ghost/{}/left_a.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/left_a.png",
                            ghost_type.as_str()
                        )))
                    })?,
                atlas
                    .get_tile(&format!("ghost/{}/left_b.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/left_b.png",
                            ghost_type.as_str()
                        )))
                    })?,
            ];
            let right_tiles = [
                atlas
                    .get_tile(&format!("ghost/{}/right_a.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/right_a.png",
                            ghost_type.as_str()
                        )))
                    })?,
                atlas
                    .get_tile(&format!("ghost/{}/right_b.png", ghost_type.as_str()))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/right_b.png",
                            ghost_type.as_str()
                        )))
                    })?,
            ];

            let normal_moving = DirectionalTiles::new(
                TileSequence::new(&up_tiles),
                TileSequence::new(&down_tiles),
                TileSequence::new(&left_tiles),
                TileSequence::new(&right_tiles),
            );
            let normal = DirectionalAnimation::new(normal_moving, normal_moving, animation::GHOST_NORMAL_SPEED);

            animations.insert(ghost_type, normal);
        }

        let (frightened, frightened_flashing) = {
            // Load frightened animation tiles (same for all ghosts)
            let frightened_blue_a = atlas
                .get_tile("ghost/frightened/blue_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/frightened/blue_a.png".to_string())))?;
            let frightened_blue_b = atlas
                .get_tile("ghost/frightened/blue_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/frightened/blue_b.png".to_string())))?;
            let frightened_white_a = atlas
                .get_tile("ghost/frightened/white_a.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/frightened/white_a.png".to_string())))?;
            let frightened_white_b = atlas
                .get_tile("ghost/frightened/white_b.png")
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("ghost/frightened/white_b.png".to_string())))?;

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
        self.world.insert_resource(DeltaTime(dt));

        // Run all systems
        self.schedule.run(&mut self.world);

        let state = self
            .world
            .get_resource::<GlobalState>()
            .expect("GlobalState could not be acquired");

        state.exit
    }

    // /// Renders pathfinding debug lines from each ghost to Pac-Man.
    // ///
    // /// Each ghost's path is drawn in its respective color with a small offset
    // /// to prevent overlapping lines.
    // fn render_pathfinding_debug<T: sdl2::render::RenderTarget>(&self, canvas: &mut Canvas<T>) -> GameResult<()> {
    //     let pacman_node = self.state.pacman.current_node_id();

    //     for ghost in self.state.ghosts.iter() {
    //         if let Ok(path) = ghost.calculate_path_to_target(&self.state.map.graph, pacman_node) {
    //             if path.len() < 2 {
    //                 continue; // Skip if path is too short
    //             }

    //             // Set the ghost's color
    //             canvas.set_draw_color(ghost.debug_color());

    //             // Calculate offset based on ghost index to prevent overlapping lines
    //             // let offset = (i as f32) * 2.0 - 3.0; // Offset range: -3.0 to 3.0

    //             // Calculate a consistent offset direction for the entire path
    //             // let first_node = self.map.graph.get_node(path[0]).unwrap();
    //             // let last_node = self.map.graph.get_node(path[path.len() - 1]).unwrap();

    //             // Use the overall direction from start to end to determine the perpendicular offset
    //             let offset = match ghost.ghost_type {
    //                 GhostType::Blinky => glam::Vec2::new(0.25, 0.5),
    //                 GhostType::Pinky => glam::Vec2::new(-0.25, -0.25),
    //                 GhostType::Inky => glam::Vec2::new(0.5, -0.5),
    //                 GhostType::Clyde => glam::Vec2::new(-0.5, 0.25),
    //             } * 5.0;

    //             // Calculate offset positions for all nodes using the same perpendicular direction
    //             let mut offset_positions = Vec::new();
    //             for &node_id in &path {
    //                 let node = self
    //                     .state
    //                     .map
    //                     .graph
    //                     .get_node(node_id)
    //                     .ok_or(crate::error::EntityError::NodeNotFound(node_id))?;
    //                 let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
    //                 offset_positions.push(pos + offset);
    //             }

    //             // Draw lines between the offset positions
    //             for window in offset_positions.windows(2) {
    //                 if let (Some(from), Some(to)) = (window.first(), window.get(1)) {
    //                     // Skip if the distance is too far (used for preventing lines between tunnel portals)
    //                     if from.distance_squared(*to) > (crate::constants::CELL_SIZE * 16).pow(2) as f32 {
    //                         continue;
    //                     }

    //                     // Draw the line
    //                     canvas
    //                         .draw_line((from.x as i32, from.y as i32), (to.x as i32, to.y as i32))
    //                         .map_err(|e| crate::error::GameError::Sdl(e.to_string()))?;
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}
