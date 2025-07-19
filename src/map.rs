use crate::constants::MapTile;
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};

pub struct Map {
    inner: [[MapTile; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize],
}

impl Map {
    pub fn new(raw_board: [&str; BOARD_HEIGHT as usize]) -> Map {
        let mut inner = [[MapTile::Empty; BOARD_HEIGHT as usize]; BOARD_WIDTH as usize];

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
                    c @ '0' | c @ '1' | c @ '2' | c @ '3' | c @ '4' => {
                        MapTile::StartingPosition(c.to_digit(10).unwrap() as u8)
                    }
                    '=' => MapTile::Empty,
                    _ => panic!("Unknown character in board: {}", character),
                };

                inner[x as usize][y as usize] = tile;
            }
        }

        Map { inner: inner }
    }

    pub fn get_tile(&self, cell: (i32, i32)) -> Option<MapTile> {
        let x = cell.0 as usize;
        let y = cell.1 as usize;

        if x >= BOARD_WIDTH as usize || y >= BOARD_HEIGHT as usize {
            return None;
        }

        Some(self.inner[x][y])
    }

    pub fn set_tile(&mut self, cell: (i32, i32), tile: MapTile) -> bool {
        let x = cell.0 as usize;
        let y = cell.1 as usize;

        if x >= BOARD_WIDTH as usize || y >= BOARD_HEIGHT as usize {
            return false;
        }

        self.inner[x][y] = tile;
        true
    }

    pub fn cell_to_pixel(cell: (u32, u32)) -> (i32, i32) {
        ((cell.0 as i32) * 24, ((cell.1 + 3) as i32) * 24)
    }
}
