//! This module contains all the constants used in the game.

use std::time::Duration;

use glam::UVec2;

/// Target frame duration for 60 FPS game loop timing.
///
/// Calculated as 1/60th of a second (≈16.67ms).
///
/// Uses integer arithmetic to avoid floating-point precision loss.
pub const LOOP_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

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

/// Bottom HUD row offset to reserve space below the game board.
///
/// The 2-cell vertical offset (16 pixels) provides space at the bottom of the
/// screen for displaying Pac-Man's lives (left) and fruit symbols (right).
pub const BOARD_BOTTOM_CELL_OFFSET: UVec2 = UVec2::new(0, 2);

/// Pixel-space equivalent of `BOARD_CELL_OFFSET` for rendering calculations.
///
/// Automatically calculated from the cell offset to maintain consistency
/// when the cell size changes. Used for positioning sprites and debug overlays.
pub const BOARD_PIXEL_OFFSET: UVec2 = UVec2::new(BOARD_CELL_OFFSET.x * CELL_SIZE, BOARD_CELL_OFFSET.y * CELL_SIZE);

/// Pixel-space equivalent of `BOARD_BOTTOM_CELL_OFFSET` for rendering calculations.
///
/// Automatically calculated from the cell offset to maintain consistency
/// when the cell size changes. Used for positioning bottom HUD elements.
pub const BOARD_BOTTOM_PIXEL_OFFSET: UVec2 =
    UVec2::new(BOARD_BOTTOM_CELL_OFFSET.x * CELL_SIZE, BOARD_BOTTOM_CELL_OFFSET.y * CELL_SIZE);

/// Animation timing constants for ghost state management
pub mod animation {
    /// Normal ghost movement animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_NORMAL_SPEED: u16 = 12;
    /// Eaten ghost (eyes) animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_EATEN_SPEED: u16 = 6;
    /// Frightened ghost animation speed (ticks per frame at 60 ticks/sec)
    pub const GHOST_FRIGHTENED_SPEED: u16 = 12;

    /// Time in ticks when frightened ghosts start flashing (2 seconds at 60 FPS)
    pub const FRIGHTENED_FLASH_START_TICKS: u32 = 120;
}
/// The size of the canvas, in pixels.
pub const CANVAS_SIZE: UVec2 = UVec2::new(
    (BOARD_CELL_SIZE.x + BOARD_CELL_OFFSET.x + BOARD_BOTTOM_CELL_OFFSET.x) * CELL_SIZE,
    (BOARD_CELL_SIZE.y + BOARD_CELL_OFFSET.y + BOARD_BOTTOM_CELL_OFFSET.y) * CELL_SIZE,
);

pub const LARGE_SCALE: f32 = 2.6;

pub const LARGE_CANVAS_SIZE: UVec2 = UVec2::new(
    (((BOARD_CELL_SIZE.x + BOARD_CELL_OFFSET.x + BOARD_BOTTOM_CELL_OFFSET.x) * CELL_SIZE) as f32 * LARGE_SCALE) as u32,
    (((BOARD_CELL_SIZE.y + BOARD_CELL_OFFSET.y + BOARD_BOTTOM_CELL_OFFSET.y) * CELL_SIZE) as f32 * LARGE_SCALE) as u32,
);

/// Collider size constants for different entity types
pub mod collider {
    use super::CELL_SIZE;

    /// Collider size for player and ghosts (1.375x cell size)
    pub const PLAYER_GHOST_SIZE: f32 = CELL_SIZE as f32 * 1.375;
    /// Collider size for pellets (0.4x cell size)
    pub const PELLET_SIZE: f32 = CELL_SIZE as f32 * 0.4;
    /// Collider size for power pellets/energizers (0.95x cell size)
    pub const POWER_PELLET_SIZE: f32 = CELL_SIZE as f32 * 0.95;
    /// Collider size for fruits (0.8x cell size)
    pub const FRUIT_SIZE: f32 = CELL_SIZE as f32 * 1.375;
}

/// UI and rendering constants
pub mod ui {
    /// Debug font size in points
    pub const DEBUG_FONT_SIZE: u16 = 12;
    /// Power pellet blink rate in ticks (at 60 FPS, 12 ticks = 0.2 seconds)
    pub const POWER_PELLET_BLINK_RATE: u32 = 12;
}

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

/// Game initialization constants
pub mod startup {
    /// Number of frames for the startup sequence (3 seconds at 60 FPS)
    pub const STARTUP_FRAMES: u32 = 60 * 3;
}

/// Game mechanics constants
pub mod mechanics {
    /// Player movement speed multiplier
    pub const PLAYER_SPEED: f32 = 1.15;
}
