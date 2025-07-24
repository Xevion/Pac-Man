//! A texture that blinks on/off for a specified number of ticks.
use glam::IVec2;
use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::texture::atlas::AtlasTexture;
use crate::texture::FrameDrawn;
use crate::{entity::direction::Direction, texture::atlas::texture_to_static};

pub struct BlinkingTexture {
    pub atlas: AtlasTexture,
    pub on_ticks: u32,
    pub off_ticks: u32,
    pub ticker: u32,
    pub visible: bool,
}

impl BlinkingTexture {
    pub fn new(
        texture: Texture<'_>,
        frame_count: u32,
        width: u32,
        height: u32,
        offset: Option<IVec2>,
        on_ticks: u32,
        off_ticks: u32,
    ) -> Self {
        BlinkingTexture {
            atlas: AtlasTexture::new(unsafe { texture_to_static(texture) }, frame_count, width, height, offset),
            on_ticks,
            off_ticks,
            ticker: 0,
            visible: true,
        }
    }

    /// Advances the blinking state by one tick.
    pub fn tick(&mut self) {
        self.ticker += 1;
        if self.visible && self.ticker >= self.on_ticks {
            self.visible = false;
            self.ticker = 0;
        } else if !self.visible && self.ticker >= self.off_ticks {
            self.visible = true;
            self.ticker = 0;
        }
    }

    pub fn set_color_modulation(&mut self, r: u8, g: u8, b: u8) {
        self.atlas.set_color_modulation(r, g, b);
    }
}

impl FrameDrawn for BlinkingTexture {
    fn render(&self, canvas: &mut Canvas<Window>, position: IVec2, direction: Direction, frame: Option<u32>) {
        if self.visible {
            self.atlas.render(canvas, position, direction, frame);
        }
    }
}
