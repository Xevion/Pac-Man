use glam::{IVec2, UVec2};
use sdl2::rect::Rect;

pub fn centered_with_size(pixel_pos: IVec2, size: UVec2) -> Rect {
    // Ensure the position doesn't cause integer overflow when centering
    let x = pixel_pos.x.saturating_sub(size.x as i32 / 2);
    let y = pixel_pos.y.saturating_sub(size.y as i32 / 2);

    Rect::new(x, y, size.x, size.y)
}
