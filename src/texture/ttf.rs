//! TTF font rendering using pre-rendered character atlas.
//!
//! This module provides efficient TTF font rendering by pre-rendering all needed
//! characters into a texture atlas at startup, avoiding expensive SDL2 font
//! surface-to-texture conversions every frame.

use glam::{UVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture, TextureCreator};

use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::collections::HashMap;

use crate::error::{GameError, TextureError};

/// Character atlas tile representing a single rendered character
#[derive(Clone, Copy, Debug)]
pub struct TtfCharTile {
    pub pos: UVec2,
    pub size: UVec2,
    pub advance: u32, // Character advance width for proportional fonts
}

/// TTF text atlas containing pre-rendered characters for efficient rendering
pub struct TtfAtlas {
    /// The texture containing all rendered characters
    texture: Texture,
    /// Mapping from character to its position and size in the atlas
    char_tiles: HashMap<char, TtfCharTile>,
    /// Cached color modulation state to avoid redundant SDL2 calls
    last_modulation: Option<Color>,
    /// Cached maximum character height
    max_char_height: u32,
}

const TTF_CHARS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,:-/()ms μµ%± ";

impl TtfAtlas {
    /// Creates a new TTF text atlas by pre-rendering all needed characters.
    ///
    /// This should be called once at startup. It renders all characters that might
    /// be used in text rendering into a single texture atlas for efficient GPU rendering.
    pub fn new(texture_creator: &TextureCreator<WindowContext>, font: &Font) -> Result<Self, GameError> {
        // Calculate character dimensions and advance widths for proportional fonts
        let mut char_tiles = HashMap::new();
        let mut max_height = 0u32;
        let mut total_width = 0u32;
        let mut char_metrics = Vec::new();

        // First pass: measure all characters
        for c in TTF_CHARS.chars() {
            if c == ' ' {
                // Handle space character specially - measure a non-space character for height
                let space_height = font.size_of("0").map_err(|e| GameError::Sdl(e.to_string()))?.1;
                let space_advance = font.size_of(" ").map_err(|e| GameError::Sdl(e.to_string()))?.0;
                char_tiles.insert(
                    c,
                    TtfCharTile {
                        pos: UVec2::ZERO,                  // Will be set during population
                        size: UVec2::new(0, space_height), // Space has no visual content
                        advance: space_advance,
                    },
                );
                max_height = max_height.max(space_height);
                char_metrics.push((c, 0, space_height, space_advance));
            } else {
                let (advance, height) = font.size_of(&c.to_string()).map_err(|e| GameError::Sdl(e.to_string()))?;
                char_tiles.insert(
                    c,
                    TtfCharTile {
                        pos: UVec2::ZERO, // Will be set during population
                        size: UVec2::new(advance, height),
                        advance,
                    },
                );
                max_height = max_height.max(height);
                total_width += advance;
                char_metrics.push((c, advance, height, advance));
            }
        }

        // Calculate atlas dimensions (pack characters horizontally for better space utilization)
        let atlas_size = UVec2::new(total_width, max_height);

        // Create atlas texture as a render target
        let mut atlas_texture = texture_creator
            .create_texture_target(None, atlas_size.x, atlas_size.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        atlas_texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        // Second pass: calculate positions
        let mut current_x = 0u32;
        for (c, width, _height, _advance) in char_metrics {
            if let Some(tile) = char_tiles.get_mut(&c) {
                tile.pos = UVec2::new(current_x, 0);
                current_x += width;
            }
        }

        Ok(Self {
            texture: atlas_texture,
            char_tiles,
            last_modulation: None,
            max_char_height: max_height,
        })
    }

    /// Renders all characters to the atlas texture using a canvas.
    /// This must be called after creation to populate the atlas.
    pub fn populate_atlas<C: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<C>,
        texture_creator: &TextureCreator<WindowContext>,
        font: &Font,
    ) -> Result<(), GameError> {
        let mut render_error: Option<GameError> = None;

        let result = canvas.with_texture_canvas(&mut self.texture, |atlas_canvas| {
            // Clear with transparent background
            atlas_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
            atlas_canvas.clear();

            for c in TTF_CHARS.chars() {
                if c == ' ' {
                    // Skip rendering space character - it has no visual content
                    continue;
                }

                // Render character to surface
                let surface = match font.render(&c.to_string()).blended(Color::WHITE) {
                    Ok(s) => s,
                    Err(e) => {
                        render_error = Some(GameError::Sdl(e.to_string()));
                        return;
                    }
                };

                // Create texture from surface
                let char_texture = match texture_creator.create_texture_from_surface(&surface) {
                    Ok(t) => t,
                    Err(e) => {
                        render_error = Some(GameError::Sdl(e.to_string()));
                        return;
                    }
                };

                // Get character tile info
                let tile = match self.char_tiles.get(&c) {
                    Some(t) => t,
                    None => {
                        render_error = Some(GameError::Sdl(format!("Character '{}' not found in atlas tiles", c)));
                        return;
                    }
                };

                // Copy character to atlas
                let dest = Rect::new(tile.pos.x as i32, tile.pos.y as i32, tile.size.x, tile.size.y);
                if let Err(e) = atlas_canvas.copy(&char_texture, None, dest) {
                    render_error = Some(GameError::Sdl(e.to_string()));
                    return;
                }
            }
        });

        // Check the result of with_texture_canvas and any render error
        if let Err(e) = result {
            return Err(GameError::Sdl(e.to_string()));
        }

        if let Some(error) = render_error {
            return Err(error);
        }

        Ok(())
    }

    /// Gets a character tile from the atlas
    pub fn get_char_tile(&self, c: char) -> Option<&TtfCharTile> {
        self.char_tiles.get(&c)
    }
}

/// TTF text renderer that uses the pre-rendered character atlas
pub struct TtfRenderer {
    scale: f32,
}

impl TtfRenderer {
    pub fn new(scale: f32) -> Self {
        Self { scale }
    }

    /// Renders a string of text at the given position with the specified color
    pub fn render_text<C: RenderTarget>(
        &self,
        canvas: &mut Canvas<C>,
        atlas: &mut TtfAtlas,
        text: &str,
        position: Vec2,
        color: Color,
    ) -> Result<(), TextureError> {
        let mut x_offset = 0.0;

        // Apply color modulation once at the beginning if needed
        if atlas.last_modulation != Some(color) {
            atlas.texture.set_color_mod(color.r, color.g, color.b);
            atlas.texture.set_alpha_mod(color.a);
            atlas.last_modulation = Some(color);
        }

        for c in text.chars() {
            // Get character tile info first to avoid borrowing conflicts
            let char_tile = atlas.get_char_tile(c);

            if let Some(char_tile) = char_tile {
                if char_tile.size.x > 0 && char_tile.size.y > 0 {
                    // Only render non-space characters
                    let dest = Rect::new(
                        (position.x + x_offset) as i32,
                        position.y as i32,
                        (char_tile.size.x as f32 * self.scale) as u32,
                        (char_tile.size.y as f32 * self.scale) as u32,
                    );

                    // Render the character directly
                    let src = Rect::new(
                        char_tile.pos.x as i32,
                        char_tile.pos.y as i32,
                        char_tile.size.x,
                        char_tile.size.y,
                    );
                    canvas.copy(&atlas.texture, src, dest).map_err(TextureError::RenderFailed)?;
                }

                // Advance by character advance width (proportional spacing)
                x_offset += char_tile.advance as f32 * self.scale;
            } else {
                // Fallback for unsupported characters - use a reasonable default
                x_offset += 8.0 * self.scale;
            }
        }

        Ok(())
    }

    /// Calculate the width of a text string in pixels
    pub fn text_width(&self, atlas: &TtfAtlas, text: &str) -> u32 {
        let mut total_width = 0u32;

        for c in text.chars() {
            if let Some(char_tile) = atlas.get_char_tile(c) {
                total_width += (char_tile.advance as f32 * self.scale) as u32;
            } else {
                // Fallback for unsupported characters
                total_width += (8.0 * self.scale) as u32;
            }
        }

        total_width
    }

    /// Calculate the height of text in pixels
    pub fn text_height(&self, atlas: &TtfAtlas) -> u32 {
        // Find the maximum height among all characters
        (atlas.max_char_height as f32 * self.scale) as u32
    }
}
