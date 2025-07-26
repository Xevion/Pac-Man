use anyhow::Result;
use glam::U16Vec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, Texture};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

#[derive(Clone)]
pub struct AtlasTile {
    pub atlas: Rc<RefCell<SpriteAtlas>>,
    pub pos: U16Vec2,
    pub size: U16Vec2,
    pub color: Option<Color>,
}

impl AtlasTile {
    pub fn render<C: RenderTarget>(&mut self, canvas: &mut Canvas<C>, dest: Rect) -> Result<()> {
        let color = self
            .color
            .unwrap_or(self.atlas.borrow().default_color.unwrap_or(Color::WHITE));
        self.render_with_color(canvas, dest, color)
    }

    pub fn render_with_color<C: RenderTarget>(&mut self, canvas: &mut Canvas<C>, dest: Rect, color: Color) -> Result<()> {
        let src = Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32);

        let mut atlas = self.atlas.borrow_mut();
        if atlas.last_modulation != Some(color) {
            atlas.texture.set_color_mod(color.r, color.g, color.b);
            atlas.last_modulation = Some(color);
        }

        canvas.copy(&atlas.texture, src, dest).map_err(anyhow::Error::msg)?;
        Ok(())
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

    pub fn get_tile(atlas: &Rc<RefCell<SpriteAtlas>>, name: &str) -> Option<AtlasTile> {
        let atlas_ref = atlas.borrow();
        atlas_ref.tiles.get(name).map(|frame| AtlasTile {
            atlas: Rc::clone(atlas),
            pos: U16Vec2::new(frame.x, frame.y),
            size: U16Vec2::new(frame.width, frame.height),
            color: None,
        })
    }

    pub fn set_color(&mut self, color: Color) {
        self.default_color = Some(color);
    }

    pub fn texture(&self) -> &Texture<'static> {
        &self.texture
    }
}

pub unsafe fn texture_to_static<'a>(texture: Texture<'a>) -> Texture<'static> {
    std::mem::transmute(texture)
}
