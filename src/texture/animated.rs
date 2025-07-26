//! This module provides a simple animation and atlas system for textures.
use anyhow::Result;
use sdl2::{pixels::Color, render::WindowCanvas};

use crate::texture::sprite::AtlasTile;

/// An animated texture using a texture atlas.
#[derive(Clone)]
pub struct AnimatedTexture {
    pub frames: Vec<AtlasTile>,
    pub ticks_per_frame: u32,
    pub ticker: u32,
    pub reversed: bool,
    pub paused: bool,
}

impl AnimatedTexture {
    pub fn new(frames: Vec<AtlasTile>, ticks_per_frame: u32) -> Self {
        AnimatedTexture {
            frames,
            ticks_per_frame,
            ticker: 0,
            reversed: false,
            paused: false,
        }
    }

    /// Advances the animation by one tick, unless paused.
    pub fn tick(&mut self) {
        if self.paused || self.ticks_per_frame == 0 {
            return;
        }

        self.ticker += 1;
    }

    pub fn current_tile(&mut self) -> &mut AtlasTile {
        if self.ticks_per_frame == 0 {
            return &mut self.frames[0];
        }
        let frame_index = (self.ticker / self.ticks_per_frame) as usize % self.frames.len();
        &mut self.frames[frame_index]
    }

    pub fn render(&mut self, canvas: &mut WindowCanvas, dest: sdl2::rect::Rect) -> Result<()> {
        let mut tile = self.current_tile();
        tile.render(canvas, dest)
    }
}
