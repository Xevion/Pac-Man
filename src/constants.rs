//! This module contains all the constants used in the game.

use glam::UVec2;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FruitType {
    Cherry,
    Strawberry,
    Orange,
    Apple,
    Melon,
    Galaxian,
    Bell,
    Key,
}

impl FruitType {
    pub const ALL: [FruitType; 8] = [
        FruitType::Cherry,
        FruitType::Strawberry,
        FruitType::Orange,
        FruitType::Apple,
        FruitType::Melon,
        FruitType::Galaxian,
        FruitType::Bell,
        FruitType::Key,
    ];

    pub fn score(self) -> u32 {
        match self {
            FruitType::Cherry => 100,
            FruitType::Strawberry => 300,
            FruitType::Orange => 500,
            FruitType::Apple => 700,
            FruitType::Melon => 1000,
            FruitType::Galaxian => 2000,
            FruitType::Bell => 3000,
            FruitType::Key => 5000,
        }
    }

    pub fn index(self) -> usize {
        match self {
            FruitType::Cherry => 0,
            FruitType::Strawberry => 1,
            FruitType::Orange => 2,
            FruitType::Apple => 3,
            FruitType::Melon => 4,
            FruitType::Galaxian => 5,
            FruitType::Bell => 6,
            FruitType::Key => 7,
        }
    }
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
    "     #.##    1     ##.#     ",
    "     #.## ###==### ##.#     ",
    "######.## #      # ##.######",
    "T     .   #2 3 4 #   .     T",
    "######.## #      # ##.######",
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
