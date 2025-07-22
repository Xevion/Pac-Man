//! This module defines the game map and provides functions for interacting with it.
use crate::constants::{MapTile, BOARD_OFFSET, CELL_SIZE};
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position(pub u32, pub u32);

impl Position {
    pub fn as_i32(&self) -> (i32, i32) {
        (self.0 as i32, self.1 as i32)
    }
    pub fn wrapping_add(&self, dx: i32, dy: i32) -> Position {
        Position((self.0 as i32 + dx) as u32, (self.1 as i32 + dy) as u32)
    }
}

/// The game map.
///
/// The map is represented as a 2D array of `MapTile`s. It also stores a copy of
/// the original map, which can be used to reset the map to its initial state.
pub struct Map {
    /// The current state of the map.
    current: [[MapTile; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize],
    /// The default state of the map.
    default: [[MapTile; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize],
}

impl Map {
    /// Creates a new `Map` instance from a raw board layout.
    ///
    /// # Arguments
    ///
    /// * `raw_board` - A 2D array of characters representing the board layout.
    pub fn new(raw_board: [&str; BOARD_HEIGHT as usize]) -> Map {
        let mut map = [[MapTile::Empty; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize];

        for y in 0..BOARD_HEIGHT as usize {
            let line = raw_board[y];

            for x in 0..BOARD_WIDTH as usize {
                if x >= line.len() {
                    break;
                }

                let i = (y * (BOARD_WIDTH as usize) + x) as usize;
                let character = line
                    .chars()
                    .nth(x as usize)
                    .unwrap_or_else(|| panic!("Could not get character at {} = ({}, {})", i, x, y));

                let tile = match character {
                    '#' => MapTile::Wall,
                    '.' => MapTile::Pellet,
                    'o' => MapTile::PowerPellet,
                    ' ' => MapTile::Empty,
                    'T' => MapTile::Tunnel,
                    c @ '0' | c @ '1' | c @ '2' | c @ '3' | c @ '4' => {
                        MapTile::StartingPosition(c.to_digit(10).unwrap() as u8)
                    }
                    '=' => MapTile::Empty,
                    _ => panic!("Unknown character in board: {}", character),
                };

                map[x as usize][y as usize] = tile;
            }
        }

        Map {
            current: map,
            default: map.clone(),
        }
    }

    /// Resets the map to its original state.
    pub fn reset(&mut self) {
        // Restore the map to its original state
        for x in 0..BOARD_WIDTH as usize {
            for y in 0..BOARD_HEIGHT as usize {
                self.current[x][y] = self.default[x][y];
            }
        }
    }

    /// Returns the tile at the given cell coordinates.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell coordinates, in grid coordinates.
    pub fn get_tile(&self, cell: (i32, i32)) -> Option<MapTile> {
        let x = cell.0 as usize;
        let y = cell.1 as usize;

        if x >= BOARD_WIDTH as usize || y >= BOARD_HEIGHT as usize {
            return None;
        }

        Some(self.current[x][y])
    }

    /// Sets the tile at the given cell coordinates.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell coordinates, in grid coordinates.
    /// * `tile` - The tile to set.
    pub fn set_tile(&mut self, cell: (i32, i32), tile: MapTile) -> bool {
        let x = cell.0 as usize;
        let y = cell.1 as usize;

        if x >= BOARD_WIDTH as usize || y >= BOARD_HEIGHT as usize {
            return false;
        }

        self.current[x][y] = tile;
        true
    }

    /// Converts cell coordinates to pixel coordinates.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell coordinates, in grid coordinates.
    pub fn cell_to_pixel(cell: (u32, u32)) -> (i32, i32) {
        (
            (cell.0 * CELL_SIZE) as i32,
            ((cell.1 + BOARD_OFFSET.1) * CELL_SIZE) as i32,
        )
    }
}
