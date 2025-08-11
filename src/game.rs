//! This module contains the main game logic and state.

use anyhow::Result;
use glam::UVec2;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sdl2::{
    image::LoadTexture,
    keyboard::Keycode,
    pixels::Color,
    render::{Canvas, RenderTarget, Texture, TextureCreator},
    video::WindowContext,
};

use crate::{
    asset::{get_asset_bytes, Asset},
    audio::Audio,
    constants::RAW_BOARD,
    entity::{
        ghost::{Ghost, GhostType},
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
    pub debug_mode: bool,

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
    ) -> Game {
        let map = Map::new(RAW_BOARD);

        let pacman_start_pos = map.find_starting_position(0).unwrap();
        let pacman_start_node = *map
            .grid_to_node
            .get(&glam::IVec2::new(pacman_start_pos.x as i32, pacman_start_pos.y as i32))
            .expect("Pac-Man starting position not found in graph");

        let atlas_bytes = get_asset_bytes(Asset::Atlas).expect("Failed to load asset");
        let atlas_texture = unsafe {
            let texture = texture_creator
                .load_texture_bytes(&atlas_bytes)
                .expect("Could not load atlas texture from asset API");
            sprite::texture_to_static(texture)
        };
        let atlas_json = get_asset_bytes(Asset::AtlasJson).expect("Failed to load asset");
        let atlas_mapper: AtlasMapper = serde_json::from_slice(&atlas_json).expect("Could not parse atlas JSON");
        let atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

        let mut map_texture = SpriteAtlas::get_tile(&atlas, "maze/full.png").expect("Failed to load map tile");
        map_texture.color = Some(Color::RGB(0x20, 0x20, 0xf9));

        let text_texture = TextTexture::new(1.0);
        let audio = Audio::new();
        let pacman = Pacman::new(&map.graph, pacman_start_node, &atlas);

        // Create ghosts at random positions
        let mut ghosts = Vec::new();
        let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
        let mut rng = SmallRng::from_os_rng();

        for &ghost_type in &ghost_types {
            // Find a random node for the ghost to start at
            let random_node = rng.random_range(0..map.graph.node_count());
            let ghost = Ghost::new(&map.graph, random_node, ghost_type, &atlas);
            ghosts.push(ghost);
        }

        Game {
            score: 0,
            map,
            pacman,
            ghosts,
            debug_mode: false,
            map_texture,
            text_texture,
            audio,
            atlas,
        }
    }

    pub fn keyboard_event(&mut self, keycode: Keycode) {
        self.pacman.handle_key(keycode);

        if keycode == Keycode::M {
            self.audio.set_mute(!self.audio.is_muted());
        }

        if keycode == Keycode::R {
            self.reset_game_state();
        }
    }

    /// Resets the game state, randomizing ghost positions and resetting Pac-Man
    fn reset_game_state(&mut self) {
        // Reset Pac-Man to starting position
        let pacman_start_pos = self.map.find_starting_position(0).unwrap();
        let pacman_start_node = *self
            .map
            .grid_to_node
            .get(&glam::IVec2::new(pacman_start_pos.x as i32, pacman_start_pos.y as i32))
            .expect("Pac-Man starting position not found in graph");

        self.pacman = Pacman::new(&self.map.graph, pacman_start_node, &self.atlas);

        // Randomize ghost positions
        let ghost_types = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde];
        let mut rng = SmallRng::from_os_rng();

        for (i, ghost) in self.ghosts.iter_mut().enumerate() {
            let random_node = rng.random_range(0..self.map.graph.node_count());
            *ghost = Ghost::new(&self.map.graph, random_node, ghost_types[i], &self.atlas);
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.pacman.tick(dt, &self.map.graph);

        // Update all ghosts
        for ghost in &mut self.ghosts {
            ghost.tick(dt, &self.map.graph);
        }
    }

    pub fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &mut Texture) -> Result<()> {
        canvas.with_texture_canvas(backbuffer, |canvas| {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            self.map.render(canvas, &mut self.atlas, &mut self.map_texture);

            // Render all ghosts
            for ghost in &self.ghosts {
                ghost.render(canvas, &mut self.atlas, &self.map.graph);
            }

            self.pacman.render(canvas, &mut self.atlas, &self.map.graph);
        })?;

        Ok(())
    }

    pub fn present_backbuffer<T: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<T>,
        backbuffer: &Texture,
        cursor_pos: glam::Vec2,
    ) -> Result<()> {
        canvas.copy(backbuffer, None, None).map_err(anyhow::Error::msg)?;
        if self.debug_mode {
            self.map
                .debug_render_with_cursor(canvas, &mut self.text_texture, &mut self.atlas, cursor_pos);
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
    fn render_pathfinding_debug<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<()> {
        let pacman_node = self.pacman.current_node_id();

        for (i, ghost) in self.ghosts.iter().enumerate() {
            if let Some(path) = ghost.calculate_path_to_target(&self.map.graph, pacman_node) {
                if path.len() < 2 {
                    continue; // Skip if path is too short
                }

                // Set the ghost's color
                canvas.set_draw_color(ghost.debug_color());

                // Calculate offset based on ghost index to prevent overlapping lines
                let offset = (i as f32) * 2.0 - 3.0; // Offset range: -3.0 to 3.0

                // Calculate a consistent offset direction for the entire path
                let first_node = self.map.graph.get_node(path[0]).unwrap();
                let last_node = self.map.graph.get_node(path[path.len() - 1]).unwrap();
                let first_pos = first_node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                let last_pos = last_node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

                // Use the overall direction from start to end to determine the perpendicular offset
                let overall_dir = (last_pos - first_pos).normalize();
                let perp_dir = glam::Vec2::new(-overall_dir.y, overall_dir.x);

                // Calculate offset positions for all nodes using the same perpendicular direction
                let mut offset_positions = Vec::new();
                for &node_id in &path {
                    let node = self.map.graph.get_node(node_id).unwrap();
                    let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                    offset_positions.push(pos + perp_dir * offset);
                }

                // Draw lines between the offset positions
                for window in offset_positions.windows(2) {
                    canvas
                        .draw_line(
                            (window[0].x as i32, window[0].y as i32),
                            (window[1].x as i32, window[1].y as i32),
                        )
                        .map_err(anyhow::Error::msg)?;
                }
            }
        }

        Ok(())
    }

    fn draw_hud<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>) -> Result<()> {
        let lives = 3;
        let score_text = format!("{:02}", self.score);
        let x_offset = 4;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        self.text_texture.set_scale(1.0);
        let _ = self.text_texture.render(
            canvas,
            &mut self.atlas,
            &format!("{lives}UP   HIGH SCORE   "),
            UVec2::new(8 * lives_offset as u32 + x_offset, y_offset),
        );
        let _ = self.text_texture.render(
            canvas,
            &mut self.atlas,
            &score_text,
            UVec2::new(8 * score_offset as u32 + x_offset, 8 + y_offset),
        );

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
