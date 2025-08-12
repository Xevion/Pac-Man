use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

use crate::entity::direction::Direction;
use crate::error::GameResult;
use crate::texture::animated::AnimatedTexture;
use crate::texture::sprite::SpriteAtlas;

#[derive(Clone)]
pub struct DirectionalAnimatedTexture {
    textures: [Option<AnimatedTexture>; 4],
    stopped_textures: [Option<AnimatedTexture>; 4],
}

impl DirectionalAnimatedTexture {
    pub fn new(textures: [Option<AnimatedTexture>; 4], stopped_textures: [Option<AnimatedTexture>; 4]) -> Self {
        Self {
            textures,
            stopped_textures,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for texture in self.textures.iter_mut().flatten() {
            texture.tick(dt);
        }
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        direction: Direction,
    ) -> GameResult<()> {
        if let Some(texture) = &self.textures[direction.as_usize()] {
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
    ) -> GameResult<()> {
        if let Some(texture) = &self.stopped_textures[direction.as_usize()] {
            texture.render(canvas, atlas, dest)
        } else {
            Ok(())
        }
    }

    /// Returns true if the texture has a direction.
    #[allow(dead_code)]
    pub fn has_direction(&self, direction: Direction) -> bool {
        self.textures[direction.as_usize()].is_some()
    }

    /// Returns true if the texture has a stopped direction.
    #[allow(dead_code)]
    pub fn has_stopped_direction(&self, direction: Direction) -> bool {
        self.stopped_textures[direction.as_usize()].is_some()
    }

    /// Returns the number of textures.
    #[allow(dead_code)]
    pub fn texture_count(&self) -> usize {
        self.textures.iter().filter(|t| t.is_some()).count()
    }

    /// Returns the number of stopped textures.
    #[allow(dead_code)]
    pub fn stopped_texture_count(&self) -> usize {
        self.stopped_textures.iter().filter(|t| t.is_some()).count()
    }
}
