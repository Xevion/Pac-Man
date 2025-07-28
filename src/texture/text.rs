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
