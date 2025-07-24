use sdl2::{render::Canvas, video::Window};

use crate::direction::Direction;

/// Trait for drawable atlas-based textures
pub trait FrameDrawn {
    fn render(&self, canvas: &mut Canvas<Window>, position: (i32, i32), direction: Direction, frame: Option<u32>);
}

pub mod animated;
pub mod atlas;
