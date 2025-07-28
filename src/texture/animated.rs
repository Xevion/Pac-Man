use anyhow::Result;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

#[derive(Clone)]
pub struct AnimatedTexture {
    tiles: Vec<AtlasTile>,
    frame_duration: f32,
    current_frame: usize,
    time_bank: f32,
}

impl AnimatedTexture {
    pub fn new(tiles: Vec<AtlasTile>, frame_duration: f32) -> Self {
        Self {
            tiles,
            frame_duration,
            current_frame: 0,
            time_bank: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.time_bank += dt;
        while self.time_bank >= self.frame_duration {
            self.time_bank -= self.frame_duration;
            self.current_frame = (self.current_frame + 1) % self.tiles.len();
        }
    }

    pub fn current_tile(&self) -> &AtlasTile {
        &self.tiles[self.current_frame]
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, dest: Rect) -> Result<()> {
        let mut tile = self.current_tile().clone();
        tile.render(canvas, atlas, dest)
    }
}
