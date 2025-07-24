//! This module defines the game map and provides functions for interacting with it.
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use rand::SeedableRng;

use crate::constants::{MapTile, BOARD_OFFSET, CELL_SIZE};
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};
use glam::{IVec2, UVec2};
use once_cell::sync::OnceCell;
use std::collections::{HashSet, VecDeque};

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
    pub fn get_tile(&self, cell: IVec2) -> Option<MapTile> {
        let x = cell.x as usize;
        let y = cell.y as usize;

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
    pub fn set_tile(&mut self, cell: IVec2, tile: MapTile) -> bool {
        let x = cell.x as usize;
        let y = cell.y as usize;

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
    pub fn cell_to_pixel(cell: UVec2) -> IVec2 {
        IVec2::new((cell.x * CELL_SIZE) as i32, ((cell.y + BOARD_OFFSET.1) * CELL_SIZE) as i32)
    }

    /// Returns a reference to a cached vector of all valid playable positions in the maze.
    /// This is computed once using a flood fill from a random pellet, and then cached.
    pub fn get_valid_playable_positions(&mut self) -> &Vec<UVec2> {
        use MapTile::*;
        static CACHE: OnceCell<Vec<UVec2>> = OnceCell::new();
        if let Some(cached) = CACHE.get() {
            return cached;
        }
        // Find a random starting pellet
        let mut pellet_positions = vec![];
        for (x, col) in self.current.iter().enumerate().take(BOARD_WIDTH as usize) {
            for (y, &cell) in col.iter().enumerate().take(BOARD_HEIGHT as usize) {
                match cell {
                    Pellet | PowerPellet => pellet_positions.push(UVec2::new(x as u32, y as u32)),
                    _ => {}
                }
            }
        }
        let mut rng = SmallRng::from_os_rng();
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
                    for offset in [IVec2::new(-1, 0), IVec2::new(1, 0), IVec2::new(0, -1), IVec2::new(0, 1)] {
                        let neighbor = (pos.as_ivec2() + offset).as_uvec2();
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
        let mut result: Vec<UVec2> = visited.into_iter().collect();
        result.sort_unstable_by_key(|v| (v.x, v.y));
        CACHE.get_or_init(|| result)
    }
}
