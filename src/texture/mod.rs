use glam::IVec2;
use sdl2::{render::Canvas, video::Window};

use crate::entity::direction::Direction;

/// Trait for drawable atlas-based textures
pub trait FrameDrawn {
    fn render(&self, canvas: &mut Canvas<Window>, position: IVec2, direction: Direction, frame: Option<u32>);
}

pub mod animated;
pub mod atlas;
