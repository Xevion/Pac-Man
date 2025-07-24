//! This module provides a simple animation and atlas system for textures.
use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::direction::Direction;
use crate::texture::atlas::AtlasTexture;
use crate::texture::FrameDrawn;

/// An animated texture using a texture atlas.
pub struct AnimatedAtlasTexture<'a> {
    pub atlas: AtlasTexture<'a>,
    pub ticks_per_frame: u32,
    pub ticker: u32,
    pub reversed: bool,
    pub paused: bool,
}

impl<'a> AnimatedAtlasTexture<'a> {
    pub fn new(
        texture: Texture<'a>,
        ticks_per_frame: u32,
        frame_count: u32,
        width: u32,
        height: u32,
        offset: Option<(i32, i32)>,
    ) -> Self {
        AnimatedAtlasTexture {
            atlas: AtlasTexture::new(texture, frame_count, width, height, offset),
            ticks_per_frame,
            ticker: 0,
            reversed: false,
            paused: false,
        }
    }

    fn current_frame(&self) -> u32 {
        self.ticker / self.ticks_per_frame
    }

    /// Advances the animation by one tick, unless paused.
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        if self.reversed {
            if self.ticker > 0 {
                self.ticker -= 1;
            }
            if self.ticker == 0 {
                self.reversed = !self.reversed;
            }
        } else {
            self.ticker += 1;
            if self.ticker + 1 == self.ticks_per_frame * self.atlas.frame_count {
                self.reversed = !self.reversed;
            }
        }
    }

    pub fn set_color_modulation(&mut self, r: u8, g: u8, b: u8) {
        self.atlas.set_color_modulation(r, g, b);
    }
}

impl<'a> FrameDrawn for AnimatedAtlasTexture<'a> {
    fn render(&self, canvas: &mut Canvas<Window>, position: (i32, i32), direction: Direction, frame: Option<u32>) {
        let frame = frame.unwrap_or_else(|| self.current_frame());
        self.atlas.render(canvas, position, direction, Some(frame));
    }
}
