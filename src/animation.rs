//! This module provides a simple animation and atlas system for textures.
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

use crate::direction::Direction;

/// Trait for drawable atlas-based textures
pub trait FrameDrawn {
    fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: Option<u32>,
    );
}

/// A texture atlas abstraction for static (non-animated) rendering.
pub struct AtlasTexture<'a> {
    pub raw_texture: Texture<'a>,
    pub offset: (i32, i32),
    pub frame_count: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl<'a> AtlasTexture<'a> {
    pub fn new(
        texture: Texture<'a>,
        frame_count: u32,
        frame_width: u32,
        frame_height: u32,
        offset: Option<(i32, i32)>,
    ) -> Self {
        AtlasTexture {
            raw_texture: texture,
            frame_count,
            frame_width,
            frame_height,
            offset: offset.unwrap_or((0, 0)),
        }
    }

    pub fn get_frame_rect(&self, frame: u32) -> Option<Rect> {
        if frame >= self.frame_count {
            return None;
        }
        Some(Rect::new(
            frame as i32 * self.frame_width as i32,
            0,
            self.frame_width,
            self.frame_height,
        ))
    }

    pub fn set_color_modulation(&mut self, r: u8, g: u8, b: u8) {
        self.raw_texture.set_color_mod(r, g, b);
    }
}

impl<'a> FrameDrawn for AtlasTexture<'a> {
    fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: Option<u32>,
    ) {
        let texture_source_frame_rect = self.get_frame_rect(frame.unwrap_or(0));
        let canvas_destination_rect = Rect::new(
            position.0 + self.offset.0,
            position.1 + self.offset.1,
            self.frame_width,
            self.frame_height,
        );
        canvas
            .copy_ex(
                &self.raw_texture,
                texture_source_frame_rect,
                Some(canvas_destination_rect),
                direction.angle(),
                None,
                false,
                false,
            )
            .expect("Could not render texture on canvas");
    }
}

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

    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_color_modulation(&mut self, r: u8, g: u8, b: u8) {
        self.atlas.set_color_modulation(r, g, b);
    }
}

impl<'a> FrameDrawn for AnimatedAtlasTexture<'a> {
    fn render(
        &mut self,
        canvas: &mut Canvas<Window>,
        position: (i32, i32),
        direction: Direction,
        frame: Option<u32>,
    ) {
        self.atlas.render(
            canvas,
            position,
            direction,
            frame.or(Some(self.current_frame())),
        );
        self.tick();
    }
}
