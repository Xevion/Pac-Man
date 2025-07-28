use glam::{IVec2, UVec2};
use sdl2::rect::Rect;

pub fn centered_with_size(pixel_pos: IVec2, size: UVec2) -> Rect {
    Rect::new(
        pixel_pos.x - size.x as i32 / 2,
        pixel_pos.y - size.y as i32 / 2,
        size.x,
        size.y,
    )
}
