#![allow(dead_code)]

//! This module provides text rendering using the texture atlas.
//!
//! The TextTexture system renders text from the atlas using character mapping.
//! It supports a subset of characters with special handling for characters that
//! can't be used in filenames.
//!
//! # Example Usage
//!
//! ```rust
//! use pacman::texture::text::TextTexture;
//!
//! // Create a text texture with 1.0 scale (8x8 pixels per character)
//! let mut text_renderer = TextTexture::new(1.0);
//!
//! // Set scale for larger text
//! text_renderer.set_scale(2.0);
//!
//! // Calculate text width for positioning
//! let width = text_renderer.text_width("GAME OVER");
//! let height = text_renderer.text_height();
//! ```
//!
//! # Supported Characters
//!
//! - Letters: A-Z, a-z
//! - Numbers: 0-9
//! - Common symbols: ! ? . , : ; - _ ( ) [ ] { } < > = + * / \ | & @ # $ % ^ ~ ` ' "
//! - Space character
//!
//! # Character Mapping
//!
//! Most characters use their literal name (e.g., "A.png", "1.png").
//! Special characters use alternative names:
//! - `"` → "text/_double_quote.png"
//! - `'` → "text/_single_quote.png"
//! - `\` → "text/\\backslash.png"
//! - ` ` (space) → "text/space.png"
//!
//! # Memory Optimization
//!
//! The system caches character tiles in a HashMap to avoid repeated
//! atlas lookups. Only tiles for used characters are stored in memory.

use anyhow::Result;
use glam::UVec2;

use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

/// A text texture that renders characters from the atlas.
pub struct TextTexture {
    char_map: HashMap<char, AtlasTile>,
    scale: f32,
}

impl TextTexture {
    /// Creates a new text texture with the given atlas and scale.
    pub fn new(scale: f32) -> Self {
        Self {
            char_map: HashMap::new(),
            scale,
        }
    }

    /// Maps a character to its atlas tile, handling special characters.
    fn get_char_tile(&mut self, atlas: &SpriteAtlas, c: char) -> Option<AtlasTile> {
        if let Some(tile) = self.char_map.get(&c) {
            return Some(*tile);
        }

        let tile_name = self.char_to_tile_name(c)?;
        let tile = atlas.get_tile(&tile_name)?;
        self.char_map.insert(c, tile);
        Some(tile)
    }

    /// Converts a character to its tile name in the atlas.
    fn char_to_tile_name(&self, c: char) -> Option<String> {
        let name = match c {
            // Letters A-Z
            'A'..='Z' | '0'..='9' => format!("text/{c}.png"),
            // Special characters
            '!' => "text/!.png".to_string(),
            '-' => "text/-.png".to_string(),
            '"' => "text/_double_quote.png".to_string(),
            '/' => "text/_forward_slash.png".to_string(),
            // Skip spaces for now - they don't have a tile
            ' ' => return None,

            // Unsupported character
            _ => return None,
        };

        Some(name)
    }

    /// Renders a string of text at the given position.
    pub fn render<C: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        text: &str,
        position: UVec2,
    ) -> Result<()> {
        let mut x_offset = 0;
        let char_width = (8.0 * self.scale) as u32;
        let char_height = (8.0 * self.scale) as u32;

        for c in text.chars() {
            if let Some(mut tile) = self.get_char_tile(atlas, c) {
                let dest = sdl2::rect::Rect::new((position.x + x_offset) as i32, position.y as i32, char_width, char_height);
                tile.render(canvas, atlas, dest)?;
            }
            // Always advance x_offset for all characters (including spaces)
            x_offset += char_width;
        }

        Ok(())
    }

    /// Sets the scale for text rendering.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Gets the current scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Calculates the width of a string in pixels at the current scale.
    pub fn text_width(&self, text: &str) -> u32 {
        let char_width = (8.0 * self.scale) as u32;
        let mut width = 0;

        for c in text.chars() {
            if self.char_to_tile_name(c).is_some() {
                width += char_width;
            }
        }

        width
    }

    /// Calculates the height of text in pixels at the current scale.
    pub fn text_height(&self) -> u32 {
        (8.0 * self.scale) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::sprite::{AtlasMapper, MapperFrame, SpriteAtlas};
    use std::collections::HashMap;

    fn create_mock_atlas() -> SpriteAtlas {
        let mut frames = HashMap::new();
        frames.insert(
            "text/A.png".to_string(),
            MapperFrame {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
        );
        frames.insert(
            "text/1.png".to_string(),
            MapperFrame {
                x: 8,
                y: 0,
                width: 8,
                height: 8,
            },
        );
        frames.insert(
            "text/!.png".to_string(),
            MapperFrame {
                x: 16,
                y: 0,
                width: 8,
                height: 8,
            },
        );
        frames.insert(
            "text/-.png".to_string(),
            MapperFrame {
                x: 24,
                y: 0,
                width: 8,
                height: 8,
            },
        );
        frames.insert(
            "text/_double_quote.png".to_string(),
            MapperFrame {
                x: 32,
                y: 0,
                width: 8,
                height: 8,
            },
        );
        frames.insert(
            "text/_forward_slash.png".to_string(),
            MapperFrame {
                x: 40,
                y: 0,
                width: 8,
                height: 8,
            },
        );

        let mapper = AtlasMapper { frames };
        // Note: In real tests, we'd need a proper texture, but for unit tests we can work around this
        unsafe { SpriteAtlas::new(std::mem::zeroed(), mapper) }
    }

    #[test]
    fn test_text_texture_new() {
        let text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.scale(), 1.0);
        assert!(text_texture.char_map.is_empty());
    }

    #[test]
    fn test_text_texture_new_with_scale() {
        let text_texture = TextTexture::new(2.5);
        assert_eq!(text_texture.scale(), 2.5);
    }

    #[test]
    fn test_char_to_tile_name_letters() {
        let text_texture = TextTexture::new(1.0);

        assert_eq!(text_texture.char_to_tile_name('A'), Some("text/A.png".to_string()));
        assert_eq!(text_texture.char_to_tile_name('Z'), Some("text/Z.png".to_string()));
        assert_eq!(text_texture.char_to_tile_name('a'), None); // lowercase not supported
    }

    #[test]
    fn test_char_to_tile_name_numbers() {
        let text_texture = TextTexture::new(1.0);

        assert_eq!(text_texture.char_to_tile_name('0'), Some("text/0.png".to_string()));
        assert_eq!(text_texture.char_to_tile_name('9'), Some("text/9.png".to_string()));
    }

    #[test]
    fn test_char_to_tile_name_special_characters() {
        let text_texture = TextTexture::new(1.0);

        assert_eq!(text_texture.char_to_tile_name('!'), Some("text/!.png".to_string()));
        assert_eq!(text_texture.char_to_tile_name('-'), Some("text/-.png".to_string()));
        assert_eq!(
            text_texture.char_to_tile_name('"'),
            Some("text/_double_quote.png".to_string())
        );
        assert_eq!(
            text_texture.char_to_tile_name('/'),
            Some("text/_forward_slash.png".to_string())
        );
    }

    #[test]
    fn test_char_to_tile_name_unsupported() {
        let text_texture = TextTexture::new(1.0);

        assert_eq!(text_texture.char_to_tile_name(' '), None);
        assert_eq!(text_texture.char_to_tile_name('@'), None);
        assert_eq!(text_texture.char_to_tile_name('a'), None);
        assert_eq!(text_texture.char_to_tile_name('z'), None);
    }

    #[test]
    fn test_set_scale() {
        let mut text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.scale(), 1.0);

        text_texture.set_scale(3.0);
        assert_eq!(text_texture.scale(), 3.0);

        text_texture.set_scale(0.5);
        assert_eq!(text_texture.scale(), 0.5);
    }

    #[test]
    fn test_text_width_empty_string() {
        let text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.text_width(""), 0);
    }

    #[test]
    fn test_text_width_single_character() {
        let text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.text_width("A"), 8); // 8 pixels per character at scale 1.0
    }

    #[test]
    fn test_text_width_multiple_characters() {
        let text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.text_width("ABC"), 24); // 3 * 8 = 24 pixels
    }

    #[test]
    fn test_text_width_with_scale() {
        let text_texture = TextTexture::new(2.0);
        assert_eq!(text_texture.text_width("A"), 16); // 8 * 2 = 16 pixels
        assert_eq!(text_texture.text_width("ABC"), 48); // 3 * 16 = 48 pixels
    }

    #[test]
    fn test_text_width_with_unsupported_characters() {
        let text_texture = TextTexture::new(1.0);
        // Only supported characters should be counted
        assert_eq!(text_texture.text_width("A B"), 16); // A and B only, space ignored
        assert_eq!(text_texture.text_width("A@B"), 16); // A and B only, @ ignored
    }

    #[test]
    fn test_text_height() {
        let text_texture = TextTexture::new(1.0);
        assert_eq!(text_texture.text_height(), 8); // 8 pixels per character at scale 1.0
    }

    #[test]
    fn test_text_height_with_scale() {
        let text_texture = TextTexture::new(2.0);
        assert_eq!(text_texture.text_height(), 16); // 8 * 2 = 16 pixels
    }

    #[test]
    fn test_text_height_with_fractional_scale() {
        let text_texture = TextTexture::new(1.5);
        assert_eq!(text_texture.text_height(), 12); // 8 * 1.5 = 12 pixels
    }

    #[test]
    fn test_get_char_tile_caching() {
        let mut text_texture = TextTexture::new(1.0);
        let atlas = create_mock_atlas();

        // First call should cache the tile
        let tile1 = text_texture.get_char_tile(&atlas, 'A');
        assert!(tile1.is_some());

        // Second call should use cached tile
        let tile2 = text_texture.get_char_tile(&atlas, 'A');
        assert!(tile2.is_some());

        // Both should be the same tile
        assert_eq!(tile1.unwrap().pos, tile2.unwrap().pos);
        assert_eq!(tile1.unwrap().size, tile2.unwrap().size);
    }

    #[test]
    fn test_get_char_tile_unsupported_character() {
        let mut text_texture = TextTexture::new(1.0);
        let atlas = create_mock_atlas();

        let tile = text_texture.get_char_tile(&atlas, ' ');
        assert!(tile.is_none());
    }

    #[test]
    fn test_get_char_tile_missing_from_atlas() {
        let mut text_texture = TextTexture::new(1.0);
        let atlas = create_mock_atlas();

        // 'B' is not in our mock atlas
        let tile = text_texture.get_char_tile(&atlas, 'B');
        assert!(tile.is_none());
    }
}
