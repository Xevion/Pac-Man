use anyhow::Result;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use thiserror::Error;

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

#[derive(Error, Debug)]
pub enum AnimatedTextureError {
    #[error("Frame duration must be positive, got {0}")]
    InvalidFrameDuration(f32),
}

#[derive(Debug, Clone)]
pub struct AnimatedTexture {
    tiles: Vec<AtlasTile>,
    frame_duration: f32,
    current_frame: usize,
    time_bank: f32,
}

impl AnimatedTexture {
    pub fn new(tiles: Vec<AtlasTile>, frame_duration: f32) -> Result<Self, AnimatedTextureError> {
        if frame_duration <= 0.0 {
            return Err(AnimatedTextureError::InvalidFrameDuration(frame_duration));
        }

        Ok(Self {
            tiles,
            frame_duration,
            current_frame: 0,
            time_bank: 0.0,
        })
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
        let mut tile = *self.current_tile();
        tile.render(canvas, atlas, dest)
    }

    /// Returns the current frame index.
    #[allow(dead_code)]
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Returns the time bank.
    #[allow(dead_code)]
    pub fn time_bank(&self) -> f32 {
        self.time_bank
    }

    /// Returns the frame duration.
    #[allow(dead_code)]
    pub fn frame_duration(&self) -> f32 {
        self.frame_duration
    }

    /// Returns the number of tiles in the animation.
    #[allow(dead_code)]
    pub fn tiles_len(&self) -> usize {
        self.tiles.len()
    }
}
