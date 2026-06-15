//! Adaptive screen layout.
//!
//! Maps the live window size to an integer render scale and the screen regions
//! the maze playfield and HUD panels occupy. The maze is always drawn at an
//! integer scale so pixel art stays crisp; leftover space becomes HUD panels.
//! The arrangement switches by orientation: landscape flanks the maze with
//! left/right panels, portrait stacks HUD bands above and below it.

use bevy_ecs::resource::Resource;
use glam::{UVec2, Vec2};
use sdl2::rect::Rect;

use crate::constants::{BOARD_BOTTOM_CELL_OFFSET, BOARD_CELL_OFFSET, BOARD_CELL_SIZE, CELL_SIZE};

/// Maze-only playfield width at scale 1 (28 cells).
const PLAYFIELD_W: u32 = BOARD_CELL_SIZE.x * CELL_SIZE;
/// Maze-only playfield height at scale 1 (31 cells).
const PLAYFIELD_H: u32 = BOARD_CELL_SIZE.y * CELL_SIZE;

/// Size of the dedicated maze texture at scale 1. The playfield renders into a
/// texture of this size in maze-local coordinates, then the compositor blits it
/// at `scale` into [`Layout::maze`].
pub const PLAYFIELD_SIZE: UVec2 = UVec2::new(PLAYFIELD_W, PLAYFIELD_H);

/// Top HUD band height at scale 1, used by the portrait stacked layout.
const TOP_HUD_H: u32 = BOARD_CELL_OFFSET.y * CELL_SIZE;
/// Bottom HUD band height at scale 1, used by the portrait stacked layout.
const BOTTOM_HUD_H: u32 = BOARD_BOTTOM_CELL_OFFSET.y * CELL_SIZE;
/// Total height of the portrait composition (top HUD + maze + bottom HUD) at scale 1.
const PORTRAIT_H: u32 = TOP_HUD_H + PLAYFIELD_H + BOTTOM_HUD_H;

/// Minimum landscape side-panel width in unscaled pixels, wide enough for
/// "HIGH SCORE". Chrome renders at `scale`, but the landscape scale formula
/// reserves `2 * MIN_PANEL_W` of unscaled width, which guarantees each panel ends
/// up at least `MIN_PANEL_W * scale` real pixels wide, so scaled content fits.
const MIN_PANEL_W: u32 = 12 * CELL_SIZE;

/// Initial desktop window size. Opens in landscape so the side-panel HUD is the
/// default presentation; the browser canvas overrides this via resize.
pub const DEFAULT_WINDOW: UVec2 = UVec2::new(1280, 720);

/// Which way the layout arranges the maze and HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Taller than wide: HUD stacks above/below the maze (classic arcade look).
    Portrait,
    /// Wider than tall: HUD flanks the maze in left/right panels.
    Landscape,
}

/// The resolved layout for the current window size.
///
/// All rects are destination rectangles in real window pixels. `maze` is where
/// the playfield texture is blitted (at `scale`); the panel rects are where HUD
/// widgets anchor. In landscape `left`/`right` are populated and `top`/`bottom`
/// are `None`; in portrait it is the reverse.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Layout {
    pub window: UVec2,
    pub scale: u32,
    pub orientation: Orientation,
    pub maze: Rect,
    pub left: Option<Rect>,
    pub right: Option<Rect>,
    pub top: Option<Rect>,
    pub bottom: Option<Rect>,
}

impl Layout {
    /// Computes the layout for a given window size. Pure: no SDL state involved.
    ///
    /// Orientation flips at square: a window at least as wide as it is tall flanks
    /// the maze with panels, otherwise the HUD stacks above and below it. Scale is
    /// the largest integer that fits the composition (maze plus the minimum HUD
    /// space) inside the window, floored at 1 so the maze never vanishes on a
    /// window too small to hold it.
    pub fn compute(window: UVec2) -> Self {
        if window.x >= window.y {
            let scale = (window.y / PLAYFIELD_H)
                .min(window.x / (PLAYFIELD_W + 2 * MIN_PANEL_W))
                .max(1);
            let maze_w = PLAYFIELD_W * scale;
            let maze_h = PLAYFIELD_H * scale;
            let maze_x = window.x.saturating_sub(maze_w) / 2;
            let maze_y = window.y.saturating_sub(maze_h) / 2;
            let right_x = maze_x + maze_w;

            Layout {
                window,
                scale,
                orientation: Orientation::Landscape,
                maze: Rect::new(maze_x as i32, maze_y as i32, maze_w, maze_h),
                left: Some(Rect::new(0, 0, maze_x, window.y)),
                right: Some(Rect::new(right_x as i32, 0, window.x.saturating_sub(right_x), window.y)),
                top: None,
                bottom: None,
            }
        } else {
            let scale = (window.x / PLAYFIELD_W).min(window.y / PORTRAIT_H).max(1);
            let column_w = PLAYFIELD_W * scale;
            let column_h = PORTRAIT_H * scale;
            let origin_x = window.x.saturating_sub(column_w) / 2;
            let origin_y = window.y.saturating_sub(column_h) / 2;

            let top_h = TOP_HUD_H * scale;
            let maze_h = PLAYFIELD_H * scale;
            let maze_y = origin_y + top_h;
            let bottom_y = maze_y + maze_h;

            Layout {
                window,
                scale,
                orientation: Orientation::Portrait,
                maze: Rect::new(origin_x as i32, maze_y as i32, column_w, maze_h),
                left: None,
                right: None,
                top: Some(Rect::new(origin_x as i32, origin_y as i32, column_w, top_h)),
                bottom: Some(Rect::new(origin_x as i32, bottom_y as i32, column_w, BOTTOM_HUD_H * scale)),
            }
        }
    }

    /// Inverse-maps a window-pixel point into maze-local pixel coordinates (the
    /// playfield texture's own `0..PLAYFIELD` space, before scaling).
    ///
    /// Translates raw mouse/touch positions back onto the maze after the
    /// compositor has scaled and offset it. Points outside the maze map to
    /// negative or out-of-range values; maze-local overlays rely on texture
    /// clipping rather than bounds-checking here.
    pub fn window_to_maze(&self, window_px: Vec2) -> Vec2 {
        let origin = Vec2::new(self.maze.x() as f32, self.maze.y() as f32);
        (window_px - origin) / self.scale as f32
    }

    /// Maps a maze-local pixel point onto the window: the inverse of
    /// [`Self::window_to_maze`], scaling by the integer render scale and offsetting
    /// by the maze's window origin. Lets the debug overlay annotate the scaled maze
    /// while drawing at the window's native resolution.
    pub fn maze_to_window(&self, maze_local: Vec2) -> Vec2 {
        let origin = Vec2::new(self.maze.x() as f32, self.maze.y() as f32);
        maze_local * self.scale as f32 + origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn landscape_orientation_and_scale() {
        let layout = Layout::compute(UVec2::new(1280, 720));
        assert_eq!(layout.orientation, Orientation::Landscape);
        // height binds: floor(720/248) = 2; width allows 3.
        assert_eq!(layout.scale, 2);
    }

    #[test]
    fn portrait_orientation_and_scale() {
        let layout = Layout::compute(UVec2::new(600, 900));
        assert_eq!(layout.orientation, Orientation::Portrait);
        // min(floor(600/224)=2, floor(900/288)=3) = 2.
        assert_eq!(layout.scale, 2);
    }

    #[test]
    fn landscape_maze_is_centered() {
        let layout = Layout::compute(UVec2::new(1280, 720));
        // 224*2 x 248*2 = 448x496, centered in 1280x720.
        assert_eq!(layout.maze, Rect::new(416, 112, 448, 496));
    }

    #[test]
    fn window_to_maze_inverts_the_compositor_transform() {
        let layout = Layout::compute(UVec2::new(1280, 720)); // maze at (416, 112), scale 2
                                                             // The maze top-left maps to the texture origin.
        assert_eq!(layout.window_to_maze(Vec2::new(416.0, 112.0)), Vec2::new(0.0, 0.0));
        // A point 20px right / 10px down of the maze origin is (10, 5) at scale 2.
        assert_eq!(layout.window_to_maze(Vec2::new(436.0, 122.0)), Vec2::new(10.0, 5.0));
    }

    #[test]
    fn maze_to_window_is_the_inverse_of_window_to_maze() {
        let layout = Layout::compute(UVec2::new(1280, 720)); // maze at (416, 112), scale 2
                                                             // The maze-local origin maps back to the maze's window origin.
        assert_eq!(layout.maze_to_window(Vec2::new(0.0, 0.0)), Vec2::new(416.0, 112.0));
        // (10, 5) maze-local scales by 2 and offsets: (10*2+416, 5*2+112) = (436, 122).
        assert_eq!(layout.maze_to_window(Vec2::new(10.0, 5.0)), Vec2::new(436.0, 122.0));
    }

    #[test]
    fn landscape_panels_flank_the_maze() {
        let layout = Layout::compute(UVec2::new(1280, 720));
        assert_eq!(layout.left, Some(Rect::new(0, 0, 416, 720)));
        assert_eq!(layout.right, Some(Rect::new(864, 0, 416, 720)));
        assert_eq!(layout.top, None);
        assert_eq!(layout.bottom, None);
    }

    #[test]
    fn portrait_stacks_hud_bands_around_the_maze() {
        let layout = Layout::compute(UVec2::new(600, 900));
        // group 448x576 centered at origin (76, 162).
        assert_eq!(layout.top, Some(Rect::new(76, 162, 448, 48)));
        assert_eq!(layout.maze, Rect::new(76, 210, 448, 496));
        assert_eq!(layout.bottom, Some(Rect::new(76, 706, 448, 32)));
        assert_eq!(layout.left, None);
        assert_eq!(layout.right, None);
    }

    #[test]
    fn scale_never_drops_below_one() {
        let layout = Layout::compute(UVec2::new(100, 100));
        assert_eq!(layout.scale, 1);
    }
}
