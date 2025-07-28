//! This module contains all the constants used in the game.

use std::time::Duration;

use glam::UVec2;

pub const LOOP_TIME: Duration = Duration::from_nanos((1_000_000_000.0 / 60.0) as u64);

/// The size of each cell, in pixels.
pub const CELL_SIZE: u32 = 8;
/// The size of the game board, in cells.
pub const BOARD_CELL_SIZE: UVec2 = UVec2::new(28, 31);

/// The scale factor for the window (integer zoom)
pub const SCALE: f32 = 2.6;

/// The offset of the game board from the top-left corner of the window, in cells.
pub const BOARD_CELL_OFFSET: UVec2 = UVec2::new(0, 3);
/// The offset of the game board from the top-left corner of the window, in pixels.
pub const BOARD_PIXEL_OFFSET: UVec2 = UVec2::new(BOARD_CELL_OFFSET.x * CELL_SIZE, BOARD_CELL_OFFSET.y * CELL_SIZE);
/// The size of the game board, in pixels.
pub const BOARD_PIXEL_SIZE: UVec2 = UVec2::new(BOARD_CELL_SIZE.x * CELL_SIZE, BOARD_CELL_SIZE.y * CELL_SIZE);
/// The size of the canvas, in pixels.
pub const CANVAS_SIZE: UVec2 = UVec2::new(
    (BOARD_CELL_SIZE.x + BOARD_CELL_OFFSET.x) * CELL_SIZE,
    (BOARD_CELL_SIZE.y + BOARD_CELL_OFFSET.y) * CELL_SIZE,
);

/// An enum representing the different types of tiles on the map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapTile {
    /// An empty tile.
    Empty,
    /// A wall tile.
    Wall,
    /// A regular pellet.
    Pellet,
    /// A power pellet.
    PowerPellet,
    /// A starting position for an entity.
    StartingPosition(u8),
    /// A tunnel tile.
    Tunnel,
}

/// The raw layout of the game board, as a 2D array of characters.
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
    "#o..##.......0 .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];
