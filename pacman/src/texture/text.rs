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
//! use sdl2::pixels::Color;
//!
//! // Create a text texture with 1.0 scale (8x8 pixels per character)
//! let mut text_renderer = TextTexture::new(1.0);
//!
//! // Set default color for all text
//! text_renderer.set_color(Color::WHITE);
//!
//! // Render text with default color
//! text_renderer.render(&mut canvas, &mut atlas, "Hello", position)?;
//!
//! // Render text with specific color
//! text_renderer.render_with_color(&mut canvas, &mut atlas, "World", position, Color::YELLOW)?;
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

use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

/// Converts a character to its tile name in the atlas.
fn char_to_tile_name(c: char) -> Option<String> {
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

/// A text texture that renders characters from the atlas.
#[derive(Debug)]
pub struct TextTexture {
    char_map: HashMap<char, AtlasTile>,
    scale: f32,
    default_color: Option<Color>,
}

impl Default for TextTexture {
    fn default() -> Self {
        Self {
            scale: 1.0,
            char_map: Default::default(),
            default_color: None,
        }
    }
}

impl TextTexture {
    /// Creates a new text texture with the given scale.
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn get_char_map(&self) -> &HashMap<char, AtlasTile> {
        &self.char_map
    }

    pub fn get_tile(&mut self, c: char, atlas: &mut SpriteAtlas) -> Result<Option<&AtlasTile>> {
        if self.char_map.contains_key(&c) {
            return Ok(self.char_map.get(&c));
        }

        if let Some(tile_name) = char_to_tile_name(c) {
            let tile = atlas.get_tile(&tile_name)?;
            self.char_map.insert(c, tile);
            Ok(self.char_map.get(&c))
        } else {
            Ok(None)
        }
    }

    /// Renders a string of text at the given position using the default color.
    pub fn render<C: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        text: &str,
        position: UVec2,
    ) -> Result<()> {
        let color = self.default_color.unwrap_or(Color::WHITE);
        self.render_with_color(canvas, atlas, text, position, color)
    }

    /// Renders a string of text at the given position with a specific color.
    pub fn render_with_color<C: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        text: &str,
        position: UVec2,
        color: Color,
    ) -> Result<()> {
        let mut x_offset = 0;
        let char_width = (8.0 * self.scale) as u32;
        let char_height = self.text_height();

        for c in text.chars() {
            // Get the tile from the char_map, or insert it if it doesn't exist
            if let Some(tile) = self.get_tile(c, atlas)? {
                // Render the tile with the specified color
                let dest = sdl2::rect::Rect::new((position.x + x_offset) as i32, position.y as i32, char_width, char_height);
                tile.render_with_color(canvas, atlas, dest, color)?;
            }

            // Always advance x_offset for all characters (including spaces)
            x_offset += char_width;
        }

        Ok(())
    }

    /// Calculates the width of a string in pixels at the current scale.
    pub fn text_width(&self, text: &str) -> u32 {
        let char_width = (8.0 * self.scale) as u32;
        let mut width = 0;

        for c in text.chars() {
            if char_to_tile_name(c).is_some() || c == ' ' {
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
