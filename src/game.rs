//! This module contains the main game logic and state.

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

use crate::constants::CANVAS_SIZE;
use crate::error::{GameError, GameResult, TextureError};
use crate::events::GameEvent;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems;
use crate::systems::blinking::Blinking;

use crate::systems::movement::{BufferedDirection, Position, Velocity};
use crate::systems::profiling::SystemId;
use crate::systems::render::RenderDirty;
use crate::systems::{
    audio::{audio_system, AudioEvent, AudioResource},
    blinking::blinking_system,
    collision::collision_system,
    components::{
        AudioState, Collider, DeltaTime, DirectionalAnimated, EntityType, Frozen, Ghost, GhostBundle, GhostCollider, GlobalState,
        ItemBundle, ItemCollider, LevelTiming, PacmanCollider, PlayerBundle, PlayerControlled, PlayerStateBundle, Renderable,
        ScoreResource, StartupSequence,
    },
    debug::{debug_render_system, DebugFontResource, DebugState, DebugTextureResource},
    ghost::{ghost_collision_system, ghost_movement_system},
    item::item_system,
    profiling::{profile, SystemTimings},
    render::{
        directional_render_system, dirty_render_system, hud_render_system, ready_visibility_system, render_system,
        BackbufferResource, MapTextureResource,
    },
};
use crate::texture::animated::AnimatedTexture;
use bevy_ecs::event::EventRegistry;
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::SystemSet;
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule};
use bevy_ecs::system::{NonSendMut, Res, ResMut};
use bevy_ecs::world::World;
use sdl2::image::LoadTexture;
use sdl2::render::{Canvas, ScaleMode, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::{
    asset::{get_asset_bytes, Asset},
    constants,
    events::GameCommand,
    map::render::MapRenderer,
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
        canvas: &'static mut Canvas<Window>,
        texture_creator: &'static mut TextureCreator<WindowContext>,
        event_pump: &'static mut EventPump,
    ) -> GameResult<Game> {
        let mut world = World::default();
        let mut schedule = Schedule::default();
        let ttf_context = Box::leak(Box::new(sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?));

        EventRegistry::register_event::<GameError>(&mut world);
        EventRegistry::register_event::<GameEvent>(&mut world);
        EventRegistry::register_event::<AudioEvent>(&mut world);

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
            .create_texture_target(None, output_size.0, output_size.1)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        debug_texture.set_scale_mode(ScaleMode::Nearest);

        let font_data = get_asset_bytes(Asset::Font)?;
        let static_font_data: &'static [u8] = Box::leak(font_data.to_vec().into_boxed_slice());
        let font_asset = RWops::from_bytes(static_font_data).map_err(|_| GameError::Sdl("Failed to load font".to_string()))?;
        let debug_font = ttf_context
            .load_font_from_rwops(font_asset, 12)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

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
        let pacman_start_node = map.start_positions.pacman;

        let mut textures = [None, None, None, None];
        let mut stopped_textures = [None, None, None, None];

        for direction in Direction::DIRECTIONS {
            let moving_prefix = match direction {
                Direction::Up => "pacman/up",
                Direction::Down => "pacman/down",
                Direction::Left => "pacman/left",
                Direction::Right => "pacman/right",
            };
            let moving_tiles = vec![
                SpriteAtlas::get_tile(&atlas, &format!("{moving_prefix}_a.png"))
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_a.png"))))?,
                SpriteAtlas::get_tile(&atlas, &format!("{moving_prefix}_b.png"))
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_b.png"))))?,
                SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
            ];

            let stopped_tiles = vec![SpriteAtlas::get_tile(&atlas, &format!("{moving_prefix}_b.png"))
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_b.png"))))?];

            textures[direction.as_usize()] = Some(AnimatedTexture::new(moving_tiles, 0.08)?);
            stopped_textures[direction.as_usize()] = Some(AnimatedTexture::new(stopped_tiles, 0.1)?);
        }

        let player = PlayerBundle {
            player: PlayerControlled,
            position: Position::Stopped { node: pacman_start_node },
            velocity: Velocity {
                speed: 1.15,
                direction: Direction::Left,
            },
            buffered_direction: BufferedDirection::None,
            sprite: Renderable {
                sprite: SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
                layer: 0,
                visible: true,
            },
            directional_animated: DirectionalAnimated {
                textures,
                stopped_textures,
            },
            entity_type: EntityType::Player,
            collider: Collider {
                size: constants::CELL_SIZE as f32 * 1.375,
            },
            pacman_collider: PacmanCollider,
        };

        // Spawn player and attach initial state bundle
        let player_entity = world.spawn(player).id();
        world.entity_mut(player_entity).insert(PlayerStateBundle::default());
        world.entity_mut(player_entity).insert(Frozen);

        world.insert_non_send_resource(atlas);
        world.insert_non_send_resource(event_pump);
        world.insert_non_send_resource(canvas);
        world.insert_non_send_resource(BackbufferResource(backbuffer));
        world.insert_non_send_resource(MapTextureResource(map_texture));
        world.insert_non_send_resource(DebugTextureResource(debug_texture));
        world.insert_non_send_resource(DebugFontResource(debug_font));
        world.insert_non_send_resource(AudioResource(audio));

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
        world.insert_resource(LevelTiming::for_level(1));

        world.add_observer(
            |event: Trigger<GameEvent>, mut state: ResMut<GlobalState>, _score: ResMut<ScoreResource>| {
                if matches!(*event, GameEvent::Command(GameCommand::Exit)) {
                    state.exit = true;
                }
            },
        );

        let input_system = profile(SystemId::Input, systems::input::input_system);
        let player_control_system = profile(SystemId::PlayerControls, systems::player::player_control_system);
        let player_movement_system = profile(SystemId::PlayerMovement, systems::player::player_movement_system);
        let startup_stage_system = profile(SystemId::Stage, systems::stage::startup_stage_system);
        let player_tunnel_slowdown_system = profile(SystemId::PlayerMovement, systems::player::player_tunnel_slowdown_system);
        let ghost_movement_system = profile(SystemId::Ghost, ghost_movement_system);
        let collision_system = profile(SystemId::Collision, collision_system);
        let ghost_collision_system = profile(SystemId::GhostCollision, ghost_collision_system);
        let item_system = profile(SystemId::Item, item_system);
        let audio_system = profile(SystemId::Audio, audio_system);
        let blinking_system = profile(SystemId::Blinking, blinking_system);
        let directional_render_system = profile(SystemId::DirectionalRender, directional_render_system);
        let dirty_render_system = profile(SystemId::DirtyRender, dirty_render_system);
        let hud_render_system = profile(SystemId::HudRender, hud_render_system);
        let render_system = profile(SystemId::Render, render_system);
        let debug_render_system = profile(SystemId::DebugRender, debug_render_system);

        let present_system = profile(
            SystemId::Present,
            |mut canvas: NonSendMut<&mut Canvas<Window>>, debug_state: Res<DebugState>, mut dirty: ResMut<RenderDirty>| {
                if dirty.0 || debug_state.enabled {
                    // Only copy backbuffer to main canvas if debug rendering is off
                    // (debug rendering draws directly to main canvas)
                    if !debug_state.enabled {
                        canvas.present();
                    }
                    dirty.0 = false;
                }
            },
        );

        schedule.add_systems((
            (
                input_system,
                player_control_system,
                player_movement_system,
                startup_stage_system,
            )
                .chain(),
            player_tunnel_slowdown_system,
            ghost_movement_system,
            (collision_system, ghost_collision_system, item_system).chain(),
            audio_system,
            blinking_system,
            ready_visibility_system,
            (
                directional_render_system,
                dirty_render_system,
                render_system,
                hud_render_system,
                debug_render_system,
                present_system,
            )
                .chain(),
        ));

        // Initialize StartupSequence as a global resource
        let ready_duration_ticks = {
            let duration = world
                .get_resource::<LevelTiming>()
                .map(|t| t.spawn_freeze_duration)
                .unwrap_or(1.5);
            (duration * 60.0) as u32 // Convert to ticks at 60 FPS
        };
        world.insert_resource(StartupSequence::new(ready_duration_ticks, 60));

        // Spawn ghosts
        Self::spawn_ghosts(&mut world)?;

        // Spawn items
        let pellet_sprite = SpriteAtlas::get_tile(world.non_send_resource::<SpriteAtlas>(), "maze/pellet.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("maze/pellet.png".to_string())))?;
        let energizer_sprite = SpriteAtlas::get_tile(world.non_send_resource::<SpriteAtlas>(), "maze/energizer.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("maze/energizer.png".to_string())))?;

        let nodes: Vec<_> = world.resource::<Map>().iter_nodes().map(|(id, tile)| (*id, *tile)).collect();

        for (node_id, tile) in nodes {
            let (item_type, sprite, size) = match tile {
                crate::constants::MapTile::Pellet => (EntityType::Pellet, pellet_sprite, constants::CELL_SIZE as f32 * 0.4),
                crate::constants::MapTile::PowerPellet => {
                    (EntityType::PowerPellet, energizer_sprite, constants::CELL_SIZE as f32 * 0.95)
                }
                _ => continue,
            };

            let mut item = world.spawn(ItemBundle {
                position: Position::Stopped { node: node_id },
                sprite: Renderable {
                    sprite,
                    layer: 1,
                    visible: true,
                },
                entity_type: item_type,
                collider: Collider { size },
                item_collider: ItemCollider,
            });

            if item_type == EntityType::PowerPellet {
                item.insert(Blinking {
                    timer: 0.0,
                    interval: 0.2,
                });
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
                let atlas = world.non_send_resource::<SpriteAtlas>();

                // Create directional animated textures for the ghost
                let mut textures = [None, None, None, None];
                let mut stopped_textures = [None, None, None, None];

                for direction in Direction::DIRECTIONS {
                    let moving_prefix = match direction {
                        Direction::Up => "up",
                        Direction::Down => "down",
                        Direction::Left => "left",
                        Direction::Right => "right",
                    };

                    let moving_tiles = vec![
                        SpriteAtlas::get_tile(atlas, &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "a"))
                            .ok_or_else(|| {
                                GameError::Texture(TextureError::AtlasTileNotFound(format!(
                                    "ghost/{}/{}_{}.png",
                                    ghost_type.as_str(),
                                    moving_prefix,
                                    "a"
                                )))
                            })?,
                        SpriteAtlas::get_tile(atlas, &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "b"))
                            .ok_or_else(|| {
                                GameError::Texture(TextureError::AtlasTileNotFound(format!(
                                    "ghost/{}/{}_{}.png",
                                    ghost_type.as_str(),
                                    moving_prefix,
                                    "b"
                                )))
                            })?,
                    ];

                    let stopped_tiles = vec![SpriteAtlas::get_tile(
                        atlas,
                        &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "a"),
                    )
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/{}_{}.png",
                            ghost_type.as_str(),
                            moving_prefix,
                            "a"
                        )))
                    })?];

                    textures[direction.as_usize()] = Some(AnimatedTexture::new(moving_tiles, 0.2)?);
                    stopped_textures[direction.as_usize()] = Some(AnimatedTexture::new(stopped_tiles, 0.1)?);
                }

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
                        visible: true,
                    },
                    directional_animated: DirectionalAnimated {
                        textures,
                        stopped_textures,
                    },
                    entity_type: EntityType::Ghost,
                    collider: Collider {
                        size: crate::constants::CELL_SIZE as f32 * 1.375,
                    },
                    ghost_collider: GhostCollider,
                }
            };

            world.spawn(ghost).insert(Frozen);
        }

        Ok(())
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
