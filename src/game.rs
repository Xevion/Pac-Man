//! This module contains the main game logic and state.

use glam::{UVec2, Vec2};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sdl2::{
    image::LoadTexture,
    keyboard::Keycode,
    pixels::Color,
    render::{Canvas, RenderTarget, Texture, TextureCreator},
    video::WindowContext,
};

use crate::error::{EntityError, GameError, GameResult, TextureError};

use crate::{
    asset::{get_asset_bytes, Asset},
    audio::Audio,
    constants::{CELL_SIZE, RAW_BOARD},
    entity::{
        collision::{Collidable, CollisionSystem, EntityId},
        ghost::{Ghost, GhostType},
        item::Item,
        pacman::Pacman,
        r#trait::Entity,
    },
    map::Map,
    texture::{
        sprite::{self, AtlasMapper, AtlasTile, SpriteAtlas},
        text::TextTexture,
    },
};

/// The main game state.
///
/// Contains all the information necessary to run the game, including
/// the game state, rendering resources, and audio.
pub struct Game {
    pub score: u32,
    pub map: Map,
    pub pacman: Pacman,
    pub ghosts: Vec<Ghost>,
    pub items: Vec<Item>,
    pub debug_mode: bool,

    // Collision system
    collision_system: CollisionSystem,
    pacman_id: EntityId,
    ghost_ids: Vec<EntityId>,
    item_ids: Vec<EntityId>,

    // Rendering resources
    atlas: SpriteAtlas,
    map_texture: AtlasTile,
    text_texture: TextTexture,

    // Audio
    pub audio: Audio,
}

impl Game {
    pub fn new(
        texture_creator: &TextureCreator<WindowContext>,
        _ttf_context: &sdl2::ttf::Sdl2TtfContext,
        _audio_subsystem: &sdl2::AudioSubsystem,
    ) -> GameResult<Game> {
        let map = Map::new(RAW_BOARD)?;

        let pacman_start_pos = map
            .find_starting_position(0)
            .ok_or_else(|| GameError::NotFound("Pac-Man starting position".to_string()))?;
        let pacman_start_node = *map
            .grid_to_node
            .get(&glam::IVec2::new(pacman_start_pos.x as i32, pacman_start_pos.y as i32))
            .ok_or_else(|| GameError::NotFound("Pac-Man starting position not found in graph".to_string()))?;

        let atlas_bytes = get_asset_bytes(Asset::Atlas)?;
        let atlas_texture = unsafe {
            let texture = texture_creator.load_texture_bytes(&atlas_bytes).map_err(|e| {
                if e.to_string().contains("format") || e.to_string().contains("unsupported") {
                    GameError::Texture(TextureError::InvalidFormat(format!("Unsupported texture format: {e}")))
                } else {
                    GameError::Texture(TextureError::LoadFailed(e.to_string()))
                }
            })?;
            sprite::texture_to_static(texture)
        };
        let atlas_json = get_asset_bytes(Asset::AtlasJson)?;
        let atlas_mapper: AtlasMapper = serde_json::from_slice(&atlas_json)?;
        let atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

        let mut map_texture = SpriteAtlas::get_tile(&atlas, "maze/full.png")
            .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("maze/full.png".to_string())))?;
        map_texture.color = Some(Color::RGB(0x20, 0x20, 0xf9));

        let text_texture = TextTexture::new(1.0);
        let audio = Audio::new();
        let pacman = Pacman::new(&map.graph, pacman_start_node, &atlas)?;

        // Generate items (pellets and energizers)
        let items = map.generate_items(&atlas)?;

        // Create ghosts at random positions
        let mut ghosts = Vec::new();
        let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
        let mut rng = SmallRng::from_os_rng();

        if map.graph.node_count() == 0 {
            return Err(GameError::Config("Game map has no nodes - invalid configuration".to_string()));
            // TODO: This is a bug, we should handle this better
        }

        for &ghost_type in &ghost_types {
            // Find a random node for the ghost to start at
            let random_node = rng.random_range(0..map.graph.node_count());
            let ghost = Ghost::new(&map.graph, random_node, ghost_type, &atlas)?;
            ghosts.push(ghost);
        }

        // Initialize collision system
        let mut collision_system = CollisionSystem::default();

        // Register Pac-Man
        let pacman_id = collision_system.register_entity(pacman.position());

        // Register items
        let mut item_ids = Vec::new();
        for item in &items {
            let item_id = collision_system.register_entity(item.position());
            item_ids.push(item_id);
        }

        // Register ghosts
        let mut ghost_ids = Vec::new();
        for ghost in &ghosts {
            let ghost_id = collision_system.register_entity(ghost.position());
            ghost_ids.push(ghost_id);
        }

        Ok(Game {
            score: 0,
            map,
            pacman,
            ghosts,
            items,
            debug_mode: false,
            collision_system,
            pacman_id,
            ghost_ids,
            item_ids,
            map_texture,
            text_texture,
            audio,
            atlas,
        })
    }

    pub fn keyboard_event(&mut self, keycode: Keycode) {
        self.pacman.handle_key(keycode);

        if keycode == Keycode::M {
            self.audio.set_mute(!self.audio.is_muted());
        }

        if keycode == Keycode::R {
            if let Err(e) = self.reset_game_state() {
                tracing::error!("Failed to reset game state: {}", e);
            }
        }
    }

    /// Resets the game state, randomizing ghost positions and resetting Pac-Man
    fn reset_game_state(&mut self) -> GameResult<()> {
        // Reset Pac-Man to starting position
        let pacman_start_pos = self
            .map
            .find_starting_position(0)
            .ok_or_else(|| GameError::NotFound("Pac-Man starting position".to_string()))?;
        let pacman_start_node = *self
            .map
            .grid_to_node
            .get(&glam::IVec2::new(pacman_start_pos.x as i32, pacman_start_pos.y as i32))
            .ok_or_else(|| GameError::NotFound("Pac-Man starting position not found in graph".to_string()))?;

        self.pacman = Pacman::new(&self.map.graph, pacman_start_node, &self.atlas)?;

        // Reset items
        self.items = self.map.generate_items(&self.atlas)?;

        // Randomize ghost positions
        let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
        let mut rng = SmallRng::from_os_rng();

        for (i, ghost) in self.ghosts.iter_mut().enumerate() {
            let random_node = rng.random_range(0..self.map.graph.node_count());
            *ghost = Ghost::new(&self.map.graph, random_node, ghost_types[i], &self.atlas)?;
        }

        // Reset collision system
        self.collision_system = CollisionSystem::default();

        // Re-register Pac-Man
        self.pacman_id = self.collision_system.register_entity(self.pacman.position());

        // Re-register items
        self.item_ids.clear();
        for item in &self.items {
            let item_id = self.collision_system.register_entity(item.position());
            self.item_ids.push(item_id);
        }

        // Re-register ghosts
        self.ghost_ids.clear();
        for ghost in &self.ghosts {
            let ghost_id = self.collision_system.register_entity(ghost.position());
            self.ghost_ids.push(ghost_id);
        }

        Ok(())
    }

    pub fn tick(&mut self, dt: f32) {
        self.pacman.tick(dt, &self.map.graph);

        // Update all ghosts
        for ghost in &mut self.ghosts {
            ghost.tick(dt, &self.map.graph);
        }

        // Update collision system positions
        self.update_collision_positions();

        // Check for collisions
        self.check_collisions();
    }

    fn update_collision_positions(&mut self) {
        // Update Pac-Man's position
        self.collision_system.update_position(self.pacman_id, self.pacman.position());

        // Update ghost positions
        for (ghost, &ghost_id) in self.ghosts.iter().zip(&self.ghost_ids) {
            self.collision_system.update_position(ghost_id, ghost.position());
        }
    }

    fn check_collisions(&mut self) {
        // Check Pac-Man vs Items
        let potential_collisions = self.collision_system.potential_collisions(&self.pacman.position());

        for entity_id in potential_collisions {
            if entity_id != self.pacman_id {
                // Check if this is an item collision
                if let Some(item_index) = self.find_item_by_id(entity_id) {
                    let item = &mut self.items[item_index];
                    if !item.is_collected() {
                        item.collect();
                        self.score += item.get_score();

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
        self.item_ids.iter().position(|&id| id == entity_id)
    }

    fn find_ghost_by_id(&self, entity_id: EntityId) -> Option<usize> {
        self.ghost_ids.iter().position(|&id| id == entity_id)
    }

    pub fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &mut Texture) -> GameResult<()> {
        canvas
            .with_texture_canvas(backbuffer, |canvas| {
                canvas.set_draw_color(Color::BLACK);
                canvas.clear();
                self.map.render(canvas, &mut self.atlas, &mut self.map_texture);

                // Render all items
                for item in &self.items {
                    if let Err(e) = item.render(canvas, &mut self.atlas, &self.map.graph) {
                        tracing::error!("Failed to render item: {}", e);
                    }
                }

                // Render all ghosts
                for ghost in &self.ghosts {
                    if let Err(e) = ghost.render(canvas, &mut self.atlas, &self.map.graph) {
                        tracing::error!("Failed to render ghost: {}", e);
                    }
                }

                if let Err(e) = self.pacman.render(canvas, &mut self.atlas, &self.map.graph) {
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
        if self.debug_mode {
            if let Err(e) = self
                .map
                .debug_render_with_cursor(canvas, &mut self.text_texture, &mut self.atlas, cursor_pos)
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
        let pacman_node = self.pacman.current_node_id();

        for ghost in self.ghosts.iter() {
            if let Ok(path) = ghost.calculate_path_to_target(&self.map.graph, pacman_node) {
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
                        if from.distance_squared(*to) > (CELL_SIZE * 16).pow(2) as f32 {
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
        let score_text = format!("{:02}", self.score);
        let x_offset = 4;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        self.text_texture.set_scale(1.0);
        if let Err(e) = self.text_texture.render(
            canvas,
            &mut self.atlas,
            &format!("{lives}UP   HIGH SCORE   "),
            UVec2::new(8 * lives_offset as u32 + x_offset, y_offset),
        ) {
            tracing::error!("Failed to render HUD text: {}", e);
        }
        if let Err(e) = self.text_texture.render(
            canvas,
            &mut self.atlas,
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
