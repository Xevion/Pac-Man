use anyhow::Result;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

use crate::entity::direction::Direction;
use crate::texture::animated::AnimatedTexture;
use crate::texture::sprite::SpriteAtlas;

#[derive(Clone)]
pub struct DirectionalAnimatedTexture {
    textures: HashMap<Direction, AnimatedTexture>,
    stopped_textures: HashMap<Direction, AnimatedTexture>,
}

impl DirectionalAnimatedTexture {
    pub fn new(textures: HashMap<Direction, AnimatedTexture>, stopped_textures: HashMap<Direction, AnimatedTexture>) -> Self {
        Self {
            textures,
            stopped_textures,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for texture in self.textures.values_mut() {
            texture.tick(dt);
        }
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        direction: Direction,
    ) -> Result<()> {
        if let Some(texture) = self.textures.get(&direction) {
            texture.render(canvas, atlas, dest)
        } else {
            Ok(())
        }
    }

    pub fn render_stopped<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        direction: Direction,
    ) -> Result<()> {
        if let Some(texture) = self.stopped_textures.get(&direction) {
            texture.render(canvas, atlas, dest)
        } else {
            Ok(())
        }
    }
}
