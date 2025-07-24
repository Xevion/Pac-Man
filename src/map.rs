//! This module defines the game map and provides functions for interacting with it.
use rand::seq::IteratorRandom;

use crate::constants::{MapTile, BOARD_OFFSET, CELL_SIZE};
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use once_cell::sync::OnceCell;
use std::collections::{HashSet, VecDeque};
use std::ops::Add;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SignedPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Add<SignedPosition> for Position {
    type Output = Position;
    fn add(self, rhs: SignedPosition) -> Self::Output {
        Position {
            x: (self.x as i32 + rhs.x) as u32,
            y: (self.y as i32 + rhs.y) as u32,
        }
    }
}

impl PartialEq<(u32, u32)> for Position {
    fn eq(&self, other: &(u32, u32)) -> bool {
        self.x == other.0 && self.y == other.1
    }
}

impl Position {
    pub fn as_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
    pub fn wrapping_add(&self, dx: i32, dy: i32) -> Position {
        Position {
            x: (self.x as i32 + dx) as u32,
            y: (self.y as i32 + dy) as u32,
        }
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

        for (y, line) in raw_board.iter().enumerate().take(BOARD_HEIGHT as usize) {
            for (x, character) in line.chars().enumerate().take(BOARD_WIDTH as usize) {
                let tile = match character {
                    '#' => MapTile::Wall,
                    '.' => MapTile::Pellet,
                    'o' => MapTile::PowerPellet,
                    ' ' => MapTile::Empty,
                    'T' => MapTile::Tunnel,
                    c @ '0' | c @ '1' | c @ '2' | c @ '3' | c @ '4' => MapTile::StartingPosition(c.to_digit(10).unwrap() as u8),
                    '=' => MapTile::Empty,
                    _ => panic!("Unknown character in board: {character}"),
                };
                map[x][y] = tile;
            }
        }

        Map {
            current: map,
            default: map,
        }
    }

    /// Resets the map to its original state.
    pub fn reset(&mut self) {
        // Restore the map to its original state
        for (x, col) in self.current.iter_mut().enumerate().take(BOARD_WIDTH as usize) {
            for (y, cell) in col.iter_mut().enumerate().take(BOARD_HEIGHT as usize) {
                *cell = self.default[x][y];
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
        ((cell.0 * CELL_SIZE) as i32, ((cell.1 + BOARD_OFFSET.1) * CELL_SIZE) as i32)
    }

    /// Returns a reference to a cached vector of all valid playable positions in the maze.
    /// This is computed once using a flood fill from a random pellet, and then cached.
    pub fn get_valid_playable_positions(&mut self) -> &Vec<Position> {
        use MapTile::*;
        static CACHE: OnceCell<Vec<Position>> = OnceCell::new();
        if let Some(cached) = CACHE.get() {
            return cached;
        }
        // Find a random starting pellet
        let mut pellet_positions = vec![];
        for (x, col) in self.current.iter().enumerate().take(BOARD_WIDTH as usize) {
            for (y, &cell) in col.iter().enumerate().take(BOARD_HEIGHT as usize) {
                match cell {
                    Pellet | PowerPellet => pellet_positions.push(Position {
                        x: x as u32,
                        y: y as u32,
                    }),
                    _ => {}
                }
            }
        }
        let mut rng = rand::rng();
        let &start = pellet_positions
            .iter()
            .choose(&mut rng)
            .expect("No pellet found for flood fill");
        // Flood fill
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        while let Some(pos) = queue.pop_front() {
            if !visited.insert(pos) {
                continue;
            }

            match self.current[pos.x as usize][pos.y as usize] {
                Empty | Pellet | PowerPellet => {
                    for offset in [
                        SignedPosition { x: -1, y: 0 },
                        SignedPosition { x: 1, y: 0 },
                        SignedPosition { x: 0, y: -1 },
                        SignedPosition { x: 0, y: 1 },
                    ] {
                        let neighbor = pos + offset;
                        if neighbor.x < BOARD_WIDTH && neighbor.y < BOARD_HEIGHT {
                            let neighbor_tile = self.current[neighbor.x as usize][neighbor.y as usize];
                            if matches!(neighbor_tile, Empty | Pellet | PowerPellet) {
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
                StartingPosition(_) | Wall | Tunnel => {}
            }
        }
        let mut result: Vec<Position> = visited.into_iter().collect();
        result.sort_unstable();
        CACHE.get_or_init(|| result)
    }
}
