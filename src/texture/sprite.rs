use anyhow::Result;
use glam::U16Vec2;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use serde::Deserialize;
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
    pub atlas: Rc<SpriteAtlas>,
    pub pos: U16Vec2,
    pub size: U16Vec2,
}

impl AtlasTile {
    pub fn render(&self, canvas: &mut WindowCanvas, dest: Rect) -> Result<()> {
        let src = Rect::new(self.pos.x as i32, self.pos.y as i32, self.size.x as u32, self.size.y as u32);
        canvas.copy(&self.atlas.texture, src, dest).map_err(anyhow::Error::msg)?;
        Ok(())
    }
}

pub struct SpriteAtlas {
    texture: Texture<'static>,
    tiles: HashMap<String, MapperFrame>,
}

impl SpriteAtlas {
    pub fn new(texture: Texture<'static>, mapper: AtlasMapper) -> Self {
        Self {
            texture,
            tiles: mapper.frames,
        }
    }

    pub fn get_tile(atlas: &Rc<SpriteAtlas>, name: &str) -> Option<AtlasTile> {
        atlas.tiles.get(name).map(|frame| AtlasTile {
            atlas: atlas.clone(),
            pos: U16Vec2::new(frame.x, frame.y),
            size: U16Vec2::new(frame.width, frame.height),
        })
    }
}

pub unsafe fn texture_to_static<'a>(texture: Texture<'a>) -> Texture<'static> {
    std::mem::transmute(texture)
}
