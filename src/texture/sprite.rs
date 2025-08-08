use anyhow::Result;
use glam::U16Vec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct AtlasMapper {
    pub frames: HashMap<String, MapperFrame>,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct MapperFrame {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct AtlasTile {
    pub pos: U16Vec2,
    pub size: U16Vec2,
    pub color: Option<Color>,
}

impl AtlasTile {
    pub fn render<C: RenderTarget>(&mut self, canvas: &mut Canvas<C>, atlas: &mut SpriteAtlas, dest: Rect) -> Result<()> {
        let color = self.color.unwrap_or(atlas.default_color.unwrap_or(Color::WHITE));
        self.render_with_color(canvas, atlas, dest, color)
    }

    pub fn render_with_color<C: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<C>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        color: Color,
    ) -> Result<()> {
        let src = Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32);

        if atlas.last_modulation != Some(color) {
            atlas.texture.set_color_mod(color.r, color.g, color.b);
            atlas.last_modulation = Some(color);
        }

        canvas.copy(&atlas.texture, src, dest).map_err(anyhow::Error::msg)?;
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

pub struct SpriteAtlas {
    texture: Texture<'static>,
    tiles: HashMap<String, MapperFrame>,
    default_color: Option<Color>,
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

    pub fn get_tile(&self, name: &str) -> Option<AtlasTile> {
        self.tiles.get(name).map(|frame| AtlasTile {
            pos: U16Vec2::new(frame.x, frame.y),
            size: U16Vec2::new(frame.width, frame.height),
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

/// Converts a `Texture` to a `Texture<'static>` using transmute.
///
/// # Safety
///
/// This function is unsafe because it uses `std::mem::transmute` to change the lifetime
/// of the texture from the original lifetime to `'static`. The caller must ensure that:
///
/// - The original `Texture` will live for the entire duration of the program
/// - No references to the original texture exist that could become invalid
/// - The texture is not dropped while still being used as a `'static` reference
///
/// This is typically used when you have a texture that you know will live for the entire
/// program duration and need to store it in a structure that requires a `'static` lifetime.
pub unsafe fn texture_to_static(texture: Texture) -> Texture<'static> {
    std::mem::transmute(texture)
}
