use anyhow::Result;
use glam::U16Vec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use std::collections::HashMap;
use tracing::debug;

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

/// A single tile within a sprite atlas, defined by its position and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
}

/// High-performance sprite atlas providing fast texture region lookups and rendering.
///
/// Combines a single large texture with metadata mapping to enable efficient
/// sprite rendering without texture switching. Caches color modulation state
/// to minimize redundant SDL2 calls and supports both named sprite lookups
/// and optional default color modulation configuration.
pub struct SpriteAtlas {
    /// The combined texture containing all sprite frames
    texture: Texture,
    /// Mapping from sprite names to their pixel coordinates within the texture
    tiles: HashMap<String, MapperFrame>,
    default_color: Option<Color>,
    /// Cached color modulation state to avoid redundant SDL2 calls
    last_modulation: Option<Color>,
}

impl SpriteAtlas {
    pub fn new(texture: Texture, mapper: AtlasMapper) -> Self {
        let tile_count = mapper.frames.len();
        let tiles = mapper.frames.into_iter().collect();

        debug!(tile_count, "Created sprite atlas");
        Self {
            texture,
            tiles,
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
    pub fn get_tile(&self, name: &str) -> Result<AtlasTile, TextureError> {
        let frame = self.tiles.get(name).ok_or_else(|| {
            debug!(tile_name = name, "Atlas tile not found");
            TextureError::AtlasTileNotFound(name.to_string())
        })?;
        Ok(AtlasTile {
            pos: frame.pos,
            size: frame.size,
            color: self.default_color,
        })
    }
}
