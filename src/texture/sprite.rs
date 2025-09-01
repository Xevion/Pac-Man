use anyhow::Result;
use glam::U16Vec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use std::collections::HashMap;

use crate::error::TextureError;

/// Atlas frame mapping data loaded from JSON metadata files.
#[derive(Clone, Debug)]
pub struct AtlasMapper {
    /// Mapping from sprite name to frame bounds within the atlas texture
    pub frames: HashMap<String, MapperFrame>,
}

#[derive(Copy, Clone, Debug)]
pub struct MapperFrame {
    pub pos: U16Vec2,
    pub size: U16Vec2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AtlasTile {
    pub pos: U16Vec2,
    pub size: U16Vec2,
    pub color: Option<Color>,
}

impl AtlasTile {
    pub fn render<C: RenderTarget>(
        &self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
    ) -> Result<(), TextureError> {
        let color = self.color.unwrap_or(atlas.default_color.unwrap_or(Color::WHITE));
        self.render_with_color(canvas, atlas, dest, color)?;
        Ok(())
    }

    pub fn render_with_color<C: RenderTarget>(
        &self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        color: Color,
    ) -> Result<(), TextureError> {
        let src = Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32);

        if atlas.last_modulation != Some(color) {
            atlas.texture.set_color_mod(color.r, color.g, color.b);
            atlas.last_modulation = Some(color);
        }

        canvas.copy(&atlas.texture, src, dest).map_err(TextureError::RenderFailed)?;
        Ok(())
    }

    /// Creates a new atlas tile.
    #[allow(dead_code)]
    pub fn new(pos: U16Vec2, size: U16Vec2, color: Option<Color>) -> Self {
        Self { pos, size, color }
    }

    /// Sets the color of the tile.
    #[allow(dead_code)]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// High-performance sprite atlas providing fast texture region lookups and rendering.
///
/// Combines a single large texture with metadata mapping to enable efficient
/// sprite rendering without texture switching. Caches color modulation state
/// to minimize redundant SDL2 calls and supports both named sprite lookups
/// and optional default color modulation configuration.
pub struct SpriteAtlas {
    /// The combined texture containing all sprite frames
    texture: Texture<'static>,
    /// Mapping from sprite names to their pixel coordinates within the texture
    tiles: HashMap<String, MapperFrame>,
    default_color: Option<Color>,
    /// Cached color modulation state to avoid redundant SDL2 calls
    last_modulation: Option<Color>,
}

impl SpriteAtlas {
    pub fn new(texture: Texture<'static>, mapper: AtlasMapper) -> Self {
        Self {
            texture,
            tiles: mapper.frames,
            default_color: None,
            last_modulation: None,
        }
    }

    /// Retrieves a sprite tile by name from the atlas with fast HashMap lookup.
    ///
    /// Returns an `AtlasTile` containing the texture coordinates and dimensions
    /// for the named sprite, or `None` if the sprite name is not found in the
    /// atlas. The returned tile can be used for immediate rendering or stored
    /// for repeated use in animations and entity sprites.
    pub fn get_tile(&self, name: &str) -> Option<AtlasTile> {
        self.tiles.get(name).map(|frame| AtlasTile {
            pos: frame.pos,
            size: frame.size,
            color: None,
        })
    }

    #[allow(dead_code)]
    pub fn set_color(&mut self, color: Color) {
        self.default_color = Some(color);
    }

    #[allow(dead_code)]
    pub fn texture(&self) -> &Texture<'static> {
        &self.texture
    }

    /// Returns the number of tiles in the atlas.
    #[allow(dead_code)]
    pub fn tiles_count(&self) -> usize {
        self.tiles.len()
    }

    /// Returns true if the atlas has a tile with the given name.
    #[allow(dead_code)]
    pub fn has_tile(&self, name: &str) -> bool {
        self.tiles.contains_key(name)
    }

    /// Returns the default color of the atlas.
    #[allow(dead_code)]
    pub fn default_color(&self) -> Option<Color> {
        self.default_color
    }
}
