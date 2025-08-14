//! This module contains the main game logic and state.

use glam::{UVec2, Vec2};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sdl2::{
    keyboard::Keycode,
    pixels::Color,
    render::{Canvas, RenderTarget, Texture, TextureCreator},
    video::WindowContext,
};

use crate::entity::r#trait::Entity;
use crate::error::{EntityError, GameError, GameResult};

use crate::entity::{
    collision::{Collidable, CollisionSystem, EntityId},
    ghost::{Ghost, GhostType},
    pacman::Pacman,
};

use crate::map::render::MapRenderer;
use crate::{constants, texture::sprite::SpriteAtlas};

pub mod state;
use state::GameState;

/// The `Game` struct is the main entry point for the game.
///
/// It contains the game's state and logic, and is responsible for
/// handling user input, updating the game state, and rendering the game.
pub struct Game {
    state: GameState,
}

impl Game {
    pub fn new(texture_creator: &'static TextureCreator<WindowContext>) -> GameResult<Game> {
        let state = GameState::new(texture_creator)?;

        Ok(Game { state })
    }

    pub fn keyboard_event(&mut self, keycode: Keycode) {
        self.state.pacman.handle_key(keycode);

        if keycode == Keycode::M {
            self.state.audio.set_mute(!self.state.audio.is_muted());
        }

        if keycode == Keycode::R {
            if let Err(e) = self.reset_game_state() {
                tracing::error!("Failed to reset game state: {}", e);
            }
        }
    }

    /// Resets the game state, randomizing ghost positions and resetting Pac-Man
    fn reset_game_state(&mut self) -> GameResult<()> {
        let pacman_start_node = self.state.map.start_positions.pacman;
        self.state.pacman = Pacman::new(&self.state.map.graph, pacman_start_node, &self.state.atlas)?;

        // Reset items
        self.state.items = self.state.map.generate_items(&self.state.atlas)?;

        // Randomize ghost positions
        let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
        let mut rng = SmallRng::from_os_rng();

        for (i, ghost) in self.state.ghosts.iter_mut().enumerate() {
            let random_node = rng.random_range(0..self.state.map.graph.node_count());
            *ghost = Ghost::new(&self.state.map.graph, random_node, ghost_types[i], &self.state.atlas)?;
        }

        // Reset collision system
        self.state.collision_system = CollisionSystem::default();

        // Re-register Pac-Man
        self.state.pacman_id = self.state.collision_system.register_entity(self.state.pacman.position());

        // Re-register items
        self.state.item_ids.clear();
        for item in &self.state.items {
            let item_id = self.state.collision_system.register_entity(item.position());
            self.state.item_ids.push(item_id);
        }

        // Re-register ghosts
        self.state.ghost_ids.clear();
        for ghost in &self.state.ghosts {
            let ghost_id = self.state.collision_system.register_entity(ghost.position());
            self.state.ghost_ids.push(ghost_id);
        }

        Ok(())
    }

    pub fn tick(&mut self, dt: f32) {
        self.state.pacman.tick(dt, &self.state.map.graph);

        // Update all ghosts
        for ghost in &mut self.state.ghosts {
            ghost.tick(dt, &self.state.map.graph);
        }

        // Update collision system positions
        self.update_collision_positions();

        // Check for collisions
        self.check_collisions();
    }

    /// Toggles the debug mode on and off.
    ///
    /// When debug mode is enabled, the game will render additional information
    /// that is useful for debugging, such as the collision grid and entity paths.
    pub fn toggle_debug_mode(&mut self) {
        self.state.debug_mode = !self.state.debug_mode;
    }

    fn update_collision_positions(&mut self) {
        // Update Pac-Man's position
        self.state
            .collision_system
            .update_position(self.state.pacman_id, self.state.pacman.position());

        // Update ghost positions
        for (ghost, &ghost_id) in self.state.ghosts.iter().zip(&self.state.ghost_ids) {
            self.state.collision_system.update_position(ghost_id, ghost.position());
        }
    }

    fn check_collisions(&mut self) {
        // Check Pac-Man vs Items
        let potential_collisions = self
            .state
            .collision_system
            .potential_collisions(&self.state.pacman.position());

        for entity_id in potential_collisions {
            if entity_id != self.state.pacman_id {
                // Check if this is an item collision
                if let Some(item_index) = self.find_item_by_id(entity_id) {
                    let item = &mut self.state.items[item_index];
                    if !item.is_collected() {
                        item.collect();
                        self.state.score += item.get_score();
                        self.state.audio.eat();

                        // Handle energizer effects
                        if matches!(item.item_type, crate::entity::item::ItemType::Energizer) {
                            // TODO: Make ghosts frightened
                            tracing::info!("Energizer collected! Ghosts should become frightened.");
                        }
                    }
                }

                // Check if this is a ghost collision
                if let Some(_ghost_index) = self.find_ghost_by_id(entity_id) {
                    // TODO: Handle Pac-Man being eaten by ghost
                    tracing::info!("Pac-Man collided with ghost!");
                }
            }
        }
    }

    fn find_item_by_id(&self, entity_id: EntityId) -> Option<usize> {
        self.state.item_ids.iter().position(|&id| id == entity_id)
    }

    fn find_ghost_by_id(&self, entity_id: EntityId) -> Option<usize> {
        self.state.ghost_ids.iter().position(|&id| id == entity_id)
    }

    pub fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &mut Texture) -> GameResult<()> {
        // Only render the map texture once and cache it
        if !self.state.map_rendered {
            let mut map_texture = self
                .state
                .texture_creator
                .create_texture_target(None, constants::CANVAS_SIZE.x, constants::CANVAS_SIZE.y)
                .map_err(|e| GameError::Sdl(e.to_string()))?;

            canvas
                .with_texture_canvas(&mut map_texture, |map_canvas| {
                    let mut map_tiles = Vec::with_capacity(35);
                    for i in 0..35 {
                        let tile_name = format!("maze/tiles/{}.png", i);
                        let tile = SpriteAtlas::get_tile(&self.state.atlas, &tile_name).unwrap();
                        map_tiles.push(tile);
                    }
                    MapRenderer::render_map(map_canvas, &mut self.state.atlas, &mut map_tiles);
                })
                .map_err(|e| GameError::Sdl(e.to_string()))?;
            self.state.map_texture = Some(map_texture);
            self.state.map_rendered = true;
        }

        canvas
            .with_texture_canvas(backbuffer, |canvas| {
                canvas.set_draw_color(Color::BLACK);
                canvas.clear();
                if let Some(ref map_texture) = self.state.map_texture {
                    canvas.copy(map_texture, None, None).unwrap();
                }

                // Render all items
                for item in &self.state.items {
                    if let Err(e) = item.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
                        tracing::error!("Failed to render item: {}", e);
                    }
                }

                // Render all ghosts
                for ghost in &self.state.ghosts {
                    if let Err(e) = ghost.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
                        tracing::error!("Failed to render ghost: {}", e);
                    }
                }

                if let Err(e) = self.state.pacman.render(canvas, &mut self.state.atlas, &self.state.map.graph) {
                    tracing::error!("Failed to render pacman: {}", e);
                }
            })
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        Ok(())
    }

    pub fn present_backbuffer<T: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<T>,
        backbuffer: &Texture,
        cursor_pos: glam::Vec2,
    ) -> GameResult<()> {
        canvas
            .copy(backbuffer, None, None)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        if self.state.debug_mode {
            if let Err(e) =
                self.state
                    .map
                    .debug_render_with_cursor(canvas, &mut self.state.text_texture, &mut self.state.atlas, cursor_pos)
            {
                tracing::error!("Failed to render debug cursor: {}", e);
            }
            self.render_pathfinding_debug(canvas)?;
        }
        self.draw_hud(canvas)?;
        canvas.present();
        Ok(())
    }

    /// Renders pathfinding debug lines from each ghost to Pac-Man.
    ///
    /// Each ghost's path is drawn in its respective color with a small offset
    /// to prevent overlapping lines.
    fn render_pathfinding_debug<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> GameResult<()> {
        let pacman_node = self.state.pacman.current_node_id();

        for ghost in self.state.ghosts.iter() {
            if let Ok(path) = ghost.calculate_path_to_target(&self.state.map.graph, pacman_node) {
                if path.len() < 2 {
                    continue; // Skip if path is too short
                }

                // Set the ghost's color
                canvas.set_draw_color(ghost.debug_color());

                // Calculate offset based on ghost index to prevent overlapping lines
                // let offset = (i as f32) * 2.0 - 3.0; // Offset range: -3.0 to 3.0

                // Calculate a consistent offset direction for the entire path
                // let first_node = self.map.graph.get_node(path[0]).unwrap();
                // let last_node = self.map.graph.get_node(path[path.len() - 1]).unwrap();

                // Use the overall direction from start to end to determine the perpendicular offset
                let offset = match ghost.ghost_type {
                    GhostType::Blinky => Vec2::new(0.25, 0.5),
                    GhostType::Pinky => Vec2::new(-0.25, -0.25),
                    GhostType::Inky => Vec2::new(0.5, -0.5),
                    GhostType::Clyde => Vec2::new(-0.5, 0.25),
                } * 5.0;

                // Calculate offset positions for all nodes using the same perpendicular direction
                let mut offset_positions = Vec::new();
                for &node_id in &path {
                    let node = self
                        .state
                        .map
                        .graph
                        .get_node(node_id)
                        .ok_or(GameError::Entity(EntityError::NodeNotFound(node_id)))?;
                    let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                    offset_positions.push(pos + offset);
                }

                // Draw lines between the offset positions
                for window in offset_positions.windows(2) {
                    if let (Some(from), Some(to)) = (window.first(), window.get(1)) {
                        // Skip if the distance is too far (used for preventing lines between tunnel portals)
                        if from.distance_squared(*to) > (crate::constants::CELL_SIZE * 16).pow(2) as f32 {
                            continue;
                        }

                        // Draw the line
                        canvas
                            .draw_line((from.x as i32, from.y as i32), (to.x as i32, to.y as i32))
                            .map_err(|e| GameError::Sdl(e.to_string()))?;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_hud<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) -> GameResult<()> {
        let lives = 3;
        let score_text = format!("{:02}", self.state.score);
        let x_offset = 4;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        self.state.text_texture.set_scale(1.0);
        if let Err(e) = self.state.text_texture.render(
            canvas,
            &mut self.state.atlas,
            &format!("{lives}UP   HIGH SCORE   "),
            UVec2::new(8 * lives_offset as u32 + x_offset, y_offset),
        ) {
            tracing::error!("Failed to render HUD text: {}", e);
        }
        if let Err(e) = self.state.text_texture.render(
            canvas,
            &mut self.state.atlas,
            &score_text,
            UVec2::new(8 * score_offset as u32 + x_offset, 8 + y_offset),
        ) {
            tracing::error!("Failed to render score text: {}", e);
        }

        // Display FPS information in top-left corner
        // let fps_text = format!("FPS: {:.1} (1s) / {:.1} (10s)", self.fps_1s, self.fps_10s);
        // self.render_text_on(
        //     canvas,
        //     &*texture_creator,
        //     &fps_text,
        //     IVec2::new(10, 10),
        //     Color::RGB(255, 255, 0), // Yellow color for FPS display
        // );

        Ok(())
    }
}
