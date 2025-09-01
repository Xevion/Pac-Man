//! This module contains all the constants used in the game.

use std::time::Duration;

use glam::UVec2;

/// Target frame duration for 60 FPS game loop timing.
///
/// Calculated as 1/60th of a second (≈16.67ms).
///
/// Written out explicitly to satisfy const-eval constraints.
pub const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000.0 / 60.0) as u64);

/// The size of each cell, in pixels.
pub const CELL_SIZE: u32 = 8;
/// The size of the game board, in cells.
pub const BOARD_CELL_SIZE: UVec2 = UVec2::new(28, 31);

/// The scale factor for the window (integer zoom)
pub const SCALE: f32 = 2.6;

/// Game board offset from window origin to reserve space for HUD elements.
///
/// The 3-cell vertical offset (24 pixels) provides space at the top of the
/// screen for score display, player lives, and other UI elements.
pub const BOARD_CELL_OFFSET: UVec2 = UVec2::new(0, 3);

/// Pixel-space equivalent of `BOARD_CELL_OFFSET` for rendering calculations.
///
/// Automatically calculated from the cell offset to maintain consistency
/// when the cell size changes. Used for positioning sprites and debug overlays.
pub const BOARD_PIXEL_OFFSET: UVec2 = UVec2::new(BOARD_CELL_OFFSET.x * CELL_SIZE, BOARD_CELL_OFFSET.y * CELL_SIZE);

/// Animation timing constants for ghost state management
pub mod animation {
    /// Normal ghost movement animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_NORMAL_SPEED: u16 = 12;
    /// Eaten ghost (eyes) animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_EATEN_SPEED: u16 = 6;
    /// Frightened ghost animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_FRIGHTENED_SPEED: u16 = 12;
    /// Frightened ghost flashing animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_FLASHING_SPEED: u16 = 9;

    /// Time in ticks when frightened ghosts start flashing (2 seconds at 60 FPS)
    pub const FRIGHTENED_FLASH_START_TICKS: u32 = 120;
}
/// The size of the canvas, in pixels.
pub const CANVAS_SIZE: UVec2 = UVec2::new(
    (BOARD_CELL_SIZE.x + BOARD_CELL_OFFSET.x) * CELL_SIZE,
    (BOARD_CELL_SIZE.y + BOARD_CELL_OFFSET.y) * CELL_SIZE,
);

/// Map tile types that define gameplay behavior and collision properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapTile {
    /// Traversable space with no collectible items
    Empty,
    Wall,
    /// Small collectible. Implicitly a traversable tile.
    Pellet,
    /// Large collectible. Implicitly a traversable tile.
    PowerPellet,
    /// Special traversable tile that connects to tunnel portals.
    Tunnel,
}

/// ASCII art representation of the classic Pac-Man maze layout.
///
/// Uses character symbols to define the game world. This layout is parsed by `MapTileParser`
/// to generate the navigable graph and collision geometry.
pub const RAW_BOARD: [&str; BOARD_CELL_SIZE.y as usize] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "     #.##### ## #####.#     ",
    "     #.##    ==    ##.#     ",
    "     #.## ######## ##.#     ",
    "######.## ######## ##.######",
    "T     .   ########   .     T",
    "######.## ######## ##.######",
    "     #.## ######## ##.#     ",
    "     #.##          ##.#     ",
    "     #.## ######## ##.#     ",
    "######.## ######## ##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......X .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];
