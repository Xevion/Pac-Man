//! This module contains the main game logic and state.

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

use crate::constants::CANVAS_SIZE;
use crate::ecs::components::{
    DeltaTime, DirectionalAnimated, GlobalState, PlayerBundle, PlayerControlled, Position, Renderable, Velocity,
};
use crate::ecs::interact::interact_system;
use crate::ecs::movement::movement_system;
use crate::ecs::render::{directional_render_system, render_system, BackbufferResource, MapTextureResource};
use crate::entity::direction::Direction;
use crate::error::{GameError, GameResult, TextureError};
use crate::input::commands::GameCommand;
use crate::map::builder::Map;
use crate::texture::animated::AnimatedTexture;
use bevy_ecs::event::EventRegistry;
use bevy_ecs::observer::Trigger;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::ResMut;
use bevy_ecs::{schedule::Schedule, world::World};
use sdl2::image::LoadTexture;
use sdl2::render::{Canvas, ScaleMode, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::asset::{get_asset_bytes, Asset};
use crate::input::{handle_input, Bindings};
use crate::map::render::MapRenderer;
use crate::{
    constants,
    texture::sprite::{AtlasMapper, SpriteAtlas},
};

use self::events::GameEvent;

pub mod events;
pub mod state;

/// The `Game` struct is the main entry point for the game.
///
/// It contains the game's state and logic, and is responsible for
/// handling user input, updating the game state, and rendering the game.
pub struct Game {
    pub world: World,
    pub schedule: Schedule,
}

impl Game {
    pub fn new(
        canvas: &'static mut Canvas<Window>,
        texture_creator: &'static mut TextureCreator<WindowContext>,
        event_pump: &'static mut EventPump,
    ) -> GameResult<Game> {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        EventRegistry::register_event::<GameError>(&mut world);
        EventRegistry::register_event::<GameEvent>(&mut world);

        let mut backbuffer = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        backbuffer.set_scale_mode(ScaleMode::Nearest);

        let mut map_texture = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        map_texture.set_scale_mode(ScaleMode::Nearest);

        // Load atlas and create map texture
        let atlas_bytes = get_asset_bytes(Asset::Atlas)?;
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
                MapRenderer::render_map(map_canvas, &mut atlas, &mut map_tiles);
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
            position: Position::AtNode(pacman_start_node),
            velocity: Velocity {
                direction: Direction::Up,
                next_direction: None,
                speed: 1.125,
            },
            sprite: Renderable {
                sprite: SpriteAtlas::get_tile(&atlas, "pacman/full.png")
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
                layer: 0,
            },
            directional_animated: DirectionalAnimated {
                textures,
                stopped_textures,
            },
        };

        world.insert_non_send_resource(atlas);
        world.insert_non_send_resource(event_pump);
        world.insert_non_send_resource(canvas);
        world.insert_non_send_resource(BackbufferResource(backbuffer));
        world.insert_non_send_resource(MapTextureResource(map_texture));

        world.insert_resource(map);
        world.insert_resource(GlobalState { exit: false });
        world.insert_resource(Bindings::default());
        world.insert_resource(DeltaTime(0f32));

        world.add_observer(|event: Trigger<GameEvent>, mut state: ResMut<GlobalState>| match *event {
            GameEvent::Command(command) => match command {
                GameCommand::Exit => {
                    state.exit = true;
                }
                _ => {}
            },
        });

        schedule.add_systems(
            (
                handle_input,
                interact_system,
                movement_system,
                directional_render_system,
                render_system,
            )
                .chain(),
        );

        // Spawn player
        world.spawn(player);

        Ok(Game { world, schedule })
    }

    // fn handle_command(&mut self, command: crate::input::commands::GameCommand) {
    //     use crate::input::commands::GameCommand;
    //     match command {
    //         GameCommand::MovePlayer(direction) => {
    //             self.state.pacman.set_next_direction(direction);
    //         }
    //         GameCommand::ToggleDebug => {
    //             self.toggle_debug_mode();
    //         }
    //         GameCommand::MuteAudio => {
    //             let is_muted = self.state.audio.is_muted();
    //             self.state.audio.set_mute(!is_muted);
    //         }
    //         GameCommand::ResetLevel => {
    //             if let Err(e) = self.reset_game_state() {
    //                 tracing::error!("Failed to reset game state: {}", e);
    //             }
    //         }
    //         GameCommand::TogglePause => {
    //             self.state.paused = !self.state.paused;
    //         }
    //         GameCommand::Exit => {}
    //     }
    // }

    // fn process_events(&mut self) {
    //     while let Some(event) = self.state.event_queue.pop_front() {
    //         match event {
    //             GameEvent::Command(command) => self.handle_command(command),
    //         }
    //     }
    // }

    // /// Resets the game state, randomizing ghost positions and resetting Pac-Man
    // fn reset_game_state(&mut self) -> GameResult<()> {
    //     let pacman_start_node = self.state.map.start_positions.pacman;
    //     self.state.pacman = Pacman::new(&self.state.map.graph, pacman_start_node, &self.state.atlas)?;

    //     // Reset items
    //     self.state.items = self.state.map.generate_items(&self.state.atlas)?;

    //     // Randomize ghost positions
    //     let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
    //     let mut rng = SmallRng::from_os_rng();

    //     for (i, ghost) in self.state.ghosts.iter_mut().enumerate() {
    //         let random_node = rng.random_range(0..self.state.map.graph.node_count());
    //         *ghost = Ghost::new(&self.state.map.graph, random_node, ghost_types[i], &self.state.atlas)?;
    //     }

    //     // Reset collision system
    //     self.state.collision_system = CollisionSystem::default();

    //     // Re-register Pac-Man
    //     self.state.pacman_id = self.state.collision_system.register_entity(self.state.pacman.position());

    //     // Re-register items
    //     self.state.item_ids.clear();
    //     for item in &self.state.items {
    //         let item_id = self.state.collision_system.register_entity(item.position());
    //         self.state.item_ids.push(item_id);
    //     }

    //     // Re-register ghosts
    //     self.state.ghost_ids.clear();
    //     for ghost in &self.state.ghosts {
    //         let ghost_id = self.state.collision_system.register_entity(ghost.position());
    //         self.state.ghost_ids.push(ghost_id);
    //     }

    //     Ok(())
    // }

    /// Ticks the game state.
    ///
    /// Returns true if the game should exit.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.world.insert_resource(DeltaTime(dt));

        // Run all systems
        self.schedule.run(&mut self.world);

        let state = self
            .world
            .get_resource::<GlobalState>()
            .expect("GlobalState could not be acquired");

        return state.exit;

        // // Process any events that have been posted (such as unpausing)
        // self.process_events();

        // // If the game is paused, we don't need to do anything beyond returning
        // if self.state.paused {
        //     return false;
        // }

        // self.schedule.run(&mut self.world);

        // self.state.pacman.tick(dt, &self.state.map.graph);

        // // Update all ghosts
        // for ghost in &mut self.state.ghosts {
        //     ghost.tick(dt, &self.state.map.graph);
        // }

        // // Update collision system positions
        // self.update_collision_positions();

        // // Check for collisions
        // self.check_collisions();
    }

    // /// Toggles the debug mode on and off.
    // ///
    // /// When debug mode is enabled, the game will render additional information
    // /// that is useful for debugging, such as the collision grid and entity paths.
    // pub fn toggle_debug_mode(&mut self) {
    //     self.state.debug_mode = !self.state.debug_mode;
    // }

    // fn update_collision_positions(&mut self) {
    //     // Update Pac-Man's position
    //     self.state
    //         .collision_system
    //         .update_position(self.state.pacman_id, self.state.pacman.position());

    //     // Update ghost positions
    //     for (ghost, &ghost_id) in self.state.ghosts.iter().zip(&self.state.ghost_ids) {
    //         self.state.collision_system.update_position(ghost_id, ghost.position());
    //     }
    // }

    // fn check_collisions(&mut self) {
    //     // Check Pac-Man vs Items
    //     let potential_collisions = self
    //         .state
    //         .collision_system
    //         .potential_collisions(&self.state.pacman.position());

    //     for entity_id in potential_collisions {
    //         if entity_id != self.state.pacman_id {
    //             // Check if this is an item collision
    //             if let Some(item_index) = self.find_item_by_id(entity_id) {
    //                 let item = &mut self.state.items[item_index];
    //                 if !item.is_collected() {
    //                     item.collect();
    //                     self.state.score += item.get_score();
    //                     self.state.audio.eat();

    //                     // Handle energizer effects
    //                     if matches!(item.item_type, crate::entity::item::ItemType::Energizer) {
    //                         // TODO: Make ghosts frightened
    //                         tracing::info!("Energizer collected! Ghosts should become frightened.");
    //                     }
    //                 }
    //             }

    //             // Check if this is a ghost collision
    //             if let Some(_ghost_index) = self.find_ghost_by_id(entity_id) {
    //                 // TODO: Handle Pac-Man being eaten by ghost
    //                 tracing::info!("Pac-Man collided with ghost!");
    //             }
    //         }
    //     }
    // }

    // fn find_item_by_id(&self, entity_id: EntityId) -> Option<usize> {
    //     self.state.item_ids.iter().position(|&id| id == entity_id)
    // }

    // fn find_ghost_by_id(&self, entity_id: EntityId) -> Option<usize> {
    //     self.state.ghost_ids.iter().position(|&id| id == entity_id)
    // }

    // pub fn draw<T: sdl2::render::RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &mut Texture) -> GameResult<()> {
    //     // Only render the map texture once and cache it
    //     if !self.state.map_rendered {
    //         let mut map_texture = self
    //             .state
    //             .texture_creator
    //             .create_texture_target(None, constants::CANVAS_SIZE.x, constants::CANVAS_SIZE.y)
    //             .map_err(|e| crate::error::GameError::Sdl(e.to_string()))?;

    //         canvas
    //             .with_texture_canvas(&mut map_texture, |map_canvas| {
    //                 let mut map_tiles = Vec::with_capacity(35);
    //                 for i in 0..35 {
    //                     let tile_name = format!("maze/tiles/{}.png", i);
    //                     let tile = SpriteAtlas::get_tile(&self.state.atlas, &tile_name).unwrap();
    //                     map_tiles.push(tile);
    //                 }
    //                 MapRenderer::render_map(map_canvas, &mut self.state.atlas, &mut map_tiles);
    //             })
    //             .map_err(|e| crate::error::GameError::Sdl(e.to_string()))?;
    //         self.state.map_texture = Some(map_texture);
    //         self.state.map_rendered = true;
    //     }

    //     canvas.set_draw_color(Color::BLACK);
    //     canvas.clear();
    //     if let Some(ref map_texture) = self.state.map_texture {
    //         canvas.copy(map_texture, None, None).unwrap();
    //     }

    //     // Render all items
    //     for item in &self.state.items {
    //         if let Err(e) = item.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
    //             tracing::error!("Failed to render item: {}", e);
    //         }
    //     }

    //     // Render all ghosts
    //     for ghost in &self.state.ghosts {
    //         if let Err(e) = ghost.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
    //             tracing::error!("Failed to render ghost: {}", e);
    //         }
    //     }

    //     if let Err(e) = self.state.pacman.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
    //         tracing::error!("Failed to render pacman: {}", e);
    //     }

    //     if self.state.debug_mode {
    //         if let Err(e) =
    //             self.state
    //                 .map
    //                 .debug_render_with_cursor(canvas, &mut self.state.text_texture, &mut self.state.atlas, cursor_pos)
    //         {
    //             tracing::error!("Failed to render debug cursor: {}", e);
    //         }
    //         self.render_pathfinding_debug(canvas)?;
    //     }
    //     self.draw_hud(canvas)?;
    //     canvas.present();

    //     Ok(())
    // }

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

    // fn draw_hud<T: sdl2::render::RenderTarget>(&mut self, canvas: &mut Canvas<T>) -> GameResult<()> {
    //     let lives = 3;
    //     let score_text = format!("{:02}", self.state.score);
    //     let x_offset = 4;
    //     let y_offset = 2;
    //     let lives_offset = 3;
    //     let score_offset = 7 - (score_text.len() as i32);
    //     self.state.text_texture.set_scale(1.0);
    //     if let Err(e) = self.state.text_texture.render(
    //         canvas,
    //         &mut self.state.atlas,
    //         &format!("{lives}UP   HIGH SCORE   "),
    //         glam::UVec2::new(8 * lives_offset as u32 + x_offset, y_offset),
    //     ) {
    //         tracing::error!("Failed to render HUD text: {}", e);
    //     }
    //     if let Err(e) = self.state.text_texture.render(
    //         canvas,
    //         &mut self.state.atlas,
    //         &score_text,
    //         glam::UVec2::new(8 * score_offset as u32 + x_offset, 8 + y_offset),
    //     ) {
    //         tracing::error!("Failed to render score text: {}", e);
    //     }

    //     // Display FPS information in top-left corner
    //     // let fps_text = format!("FPS: {:.1} (1s) / {:.1} (10s)", self.fps_1s, self.fps_10s);
    //     // self.render_text_on(
    //     //     canvas,
    //     //     &*texture_creator,
    //     //     &fps_text,
    //     //     IVec2::new(10, 10),
    //     //     Color::RGB(255, 255, 0), // Yellow color for FPS display
    //     // );

    //     Ok(())
    // }
}
