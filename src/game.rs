//! This module contains the main game logic and state.

use anyhow::Result;
use glam::UVec2;
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
    entity::pacman::Pacman,
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

        Game {
            score: 0,
            map,
            pacman,
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
    }

    pub fn tick(&mut self, dt: f32) {
        self.pacman.tick(dt, &self.map.graph);
    }

    pub fn draw<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &mut Texture) -> Result<()> {
        canvas.with_texture_canvas(backbuffer, |canvas| {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            self.map.render(canvas, &mut self.atlas, &mut self.map_texture);
            self.pacman.render(canvas, &mut self.atlas, &self.map.graph);
        })?;

        Ok(())
    }

    pub fn present_backbuffer<T: RenderTarget>(&mut self, canvas: &mut Canvas<T>, backbuffer: &Texture) -> Result<()> {
        canvas.copy(backbuffer, None, None).map_err(anyhow::Error::msg)?;
        if self.debug_mode {
            self.map.debug_render_nodes(canvas);
        }
        self.draw_hud(canvas)?;
        canvas.present();
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

#[cfg(test)]
mod tests {
    use super::*;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::Color;

    fn create_test_game() -> Game {
        // Create a minimal test game without SDL dependencies
        // This is a simplified version for testing basic logic
        let map = Map::new(RAW_BOARD);
        let pacman_start_pos = map.find_starting_position(0).unwrap();
        let pacman_start_node = *map
            .grid_to_node
            .get(&glam::IVec2::new(pacman_start_pos.x as i32, pacman_start_pos.y as i32))
            .expect("Pac-Man starting position not found in graph");

        // Create a dummy atlas for testing
        let mut mapper = std::collections::HashMap::new();
        mapper.insert(
            "pacman/up_a.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 0,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/up_b.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 16,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/down_a.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 32,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/down_b.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 48,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/left_a.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 64,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/left_b.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 80,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/right_a.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 96,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/right_b.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 112,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "pacman/full.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 128,
                y: 0,
                width: 16,
                height: 16,
            },
        );
        mapper.insert(
            "maze/full.png".to_string(),
            crate::texture::sprite::MapperFrame {
                x: 0,
                y: 0,
                width: 224,
                height: 248,
            },
        );

        let atlas_mapper = crate::texture::sprite::AtlasMapper { frames: mapper };
        let dummy_texture = unsafe { std::mem::zeroed() };
        let atlas = crate::texture::sprite::SpriteAtlas::new(dummy_texture, atlas_mapper);

        let mut map_texture = crate::texture::sprite::SpriteAtlas::get_tile(&atlas, "maze/full.png").unwrap();
        map_texture.color = Some(Color::RGB(0x20, 0x20, 0xf9));

        let text_texture = TextTexture::new(1.0);
        let audio = Audio::new();
        let pacman = Pacman::new(&map.graph, pacman_start_node, &atlas);

        Game {
            score: 0,
            map,
            pacman,
            debug_mode: false,
            map_texture,
            text_texture,
            audio,
            atlas,
        }
    }

    #[test]
    fn test_game_keyboard_event_direction_keys() {
        let mut game = create_test_game();

        // Test that direction keys are handled
        game.keyboard_event(Keycode::Up);
        game.keyboard_event(Keycode::Down);
        game.keyboard_event(Keycode::Left);
        game.keyboard_event(Keycode::Right);

        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_game_keyboard_event_mute_toggle() {
        let mut game = create_test_game();

        let initial_mute_state = game.audio.is_muted();

        // Toggle mute
        game.keyboard_event(Keycode::M);

        // Mute state should have changed
        assert_eq!(game.audio.is_muted(), !initial_mute_state);

        // Toggle again
        game.keyboard_event(Keycode::M);

        // Should be back to original state
        assert_eq!(game.audio.is_muted(), initial_mute_state);
    }

    #[test]
    fn test_game_tick() {
        let mut game = create_test_game();

        // Test that tick doesn't panic
        game.tick(0.016); // 60 FPS frame time

        assert!(true);
    }

    #[test]
    fn test_game_initial_state() {
        let game = create_test_game();

        assert_eq!(game.score, 0);
        assert!(!game.debug_mode);
        assert!(game.map.graph.node_count() > 0);
    }

    #[test]
    fn test_game_debug_mode_toggle() {
        let mut game = create_test_game();

        assert!(!game.debug_mode);

        // Toggle debug mode (this would normally be done via Space key in the app)
        game.debug_mode = !game.debug_mode;

        assert!(game.debug_mode);
    }

    #[test]
    fn test_game_score_increment() {
        let mut game = create_test_game();

        let initial_score = game.score;
        game.score += 10;

        assert_eq!(game.score, initial_score + 10);
    }

    #[test]
    fn test_game_pacman_initialization() {
        let game = create_test_game();

        // Check that Pac-Man was initialized
        assert_eq!(game.pacman.traverser.direction, crate::entity::direction::Direction::Left);
        // The traverser might start moving immediately, so we just check the direction
        assert_eq!(game.pacman.traverser.direction, crate::entity::direction::Direction::Left);
    }

    #[test]
    fn test_game_map_initialization() {
        let game = create_test_game();

        // Check that map was initialized
        assert!(game.map.graph.node_count() > 0);
        assert!(!game.map.grid_to_node.is_empty());

        // Check that Pac-Man's starting position exists
        let pacman_pos = game.map.find_starting_position(0);
        assert!(pacman_pos.is_some());
    }
}
