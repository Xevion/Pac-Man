use glam::IVec2;
use sdl2::{
    rect::Rect,
    render::{Canvas, Texture},
    video::Window,
};

use crate::{entity::direction::Direction, texture::FrameDrawn};

/// Unsafely converts a Texture with any lifetime to a 'static lifetime.
/// Only use this if you guarantee the renderer/context will never be dropped!
pub unsafe fn texture_to_static<'a>(texture: Texture<'a>) -> Texture<'static> {
    std::mem::transmute::<Texture<'a>, Texture<'static>>(texture)
}

/// A texture atlas abstraction for static (non-animated) rendering.
pub struct AtlasTexture {
    pub raw_texture: Texture<'static>,
    pub offset: IVec2,
    pub frame_count: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl AtlasTexture {
    pub fn new(texture: Texture<'static>, frame_count: u32, frame_width: u32, frame_height: u32, offset: Option<IVec2>) -> Self {
        AtlasTexture {
            raw_texture: texture,
            frame_count,
            frame_width,
            frame_height,
            offset: offset.unwrap_or(IVec2::new(0, 0)).into(),
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

impl FrameDrawn for AtlasTexture {
    fn render(&self, canvas: &mut Canvas<Window>, position: IVec2, direction: Direction, frame: Option<u32>) {
        let texture_source_frame_rect = self.get_frame_rect(frame.unwrap_or(0));
        let canvas_destination_rect = Rect::new(
            position.x + self.offset.x,
            position.y + self.offset.y,
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
