//! A texture that blinks on/off for a specified number of ticks.
use anyhow::Result;
use sdl2::render::WindowCanvas;

use crate::texture::animated::AnimatedTexture;

#[derive(Clone)]
pub struct BlinkingTexture {
    pub animation: AnimatedTexture,
    pub on_ticks: u32,
    pub off_ticks: u32,
    pub ticker: u32,
    pub visible: bool,
}

impl BlinkingTexture {
    pub fn new(animation: AnimatedTexture, on_ticks: u32, off_ticks: u32) -> Self {
        BlinkingTexture {
            animation,
            on_ticks,
            off_ticks,
            ticker: 0,
            visible: true,
        }
    }

    /// Advances the blinking state by one tick.
    pub fn tick(&mut self) {
        self.animation.tick();
        self.ticker += 1;
        if self.visible && self.ticker >= self.on_ticks {
            self.visible = false;
            self.ticker = 0;
        } else if !self.visible && self.ticker >= self.off_ticks {
            self.visible = true;
            self.ticker = 0;
        }
    }

    /// Renders the blinking texture.
    pub fn render(&self, canvas: &mut WindowCanvas, dest: sdl2::rect::Rect) -> Result<()> {
        if self.visible {
            self.animation.render(canvas, dest)
        } else {
            Ok(())
        }
    }
}
