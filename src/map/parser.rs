//! Map parsing functionality for converting raw board layouts into structured data.

use crate::constants::{MapTile, BOARD_CELL_SIZE};
use glam::IVec2;
use thiserror::Error;

/// Error type for map parsing operations.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unknown character in board: {0}")]
    UnknownCharacter(char),
    #[error("House door must have exactly 2 positions, found {0}")]
    InvalidHouseDoorCount(usize),
}

/// Represents the parsed data from a raw board layout.
#[derive(Debug)]
pub struct ParsedMap {
    /// The parsed tile layout.
    pub tiles: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// The positions of the house door tiles.
    pub house_door: [Option<IVec2>; 2],
    /// The positions of the tunnel end tiles.
    pub tunnel_ends: [Option<IVec2>; 2],
    /// Pac-Man's starting position.
    pub pacman_start: Option<IVec2>,
}

/// Parser for converting raw board layouts into structured map data.
pub struct MapTileParser;

impl MapTileParser {
    /// Parses a single character into a map tile.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to parse
    ///
    /// # Returns
    ///
    /// The parsed map tile, or an error if the character is unknown.
    pub fn parse_character(c: char) -> Result<MapTile, ParseError> {
        match c {
            '#' => Ok(MapTile::Wall),
            '.' => Ok(MapTile::Pellet),
            'o' => Ok(MapTile::PowerPellet),
            ' ' => Ok(MapTile::Empty),
            'T' => Ok(MapTile::Tunnel),
            'X' => Ok(MapTile::Empty), // Pac-Man's starting position, treated as empty
            '=' => Ok(MapTile::Wall),  // House door is represented as a wall tile
            _ => Err(ParseError::UnknownCharacter(c)),
        }
    }

    /// Parses a raw board layout into structured map data.
    ///
    /// # Arguments
    ///
    /// * `raw_board` - The raw board layout as an array of strings
    ///
    /// # Returns
    ///
    /// The parsed map data, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the board contains unknown characters or if the house door
    /// is not properly defined by exactly two '=' characters.
    pub fn parse_board(raw_board: [&str; BOARD_CELL_SIZE.y as usize]) -> Result<ParsedMap, ParseError> {
        let mut tiles = [[MapTile::Empty; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize];
        let mut house_door = [None; 2];
        let mut tunnel_ends = [None; 2];
        let mut pacman_start: Option<IVec2> = None;

        for (y, line) in raw_board.iter().enumerate().take(BOARD_CELL_SIZE.y as usize) {
            for (x, character) in line.chars().enumerate().take(BOARD_CELL_SIZE.x as usize) {
                let tile = Self::parse_character(character)?;

                // Track special positions
                match tile {
                    MapTile::Tunnel => {
                        if tunnel_ends[0].is_none() {
                            tunnel_ends[0] = Some(IVec2::new(x as i32, y as i32));
                        } else {
                            tunnel_ends[1] = Some(IVec2::new(x as i32, y as i32));
                        }
                    }
                    MapTile::Wall if character == '=' => {
                        if house_door[0].is_none() {
                            house_door[0] = Some(IVec2::new(x as i32, y as i32));
                        } else {
                            house_door[1] = Some(IVec2::new(x as i32, y as i32));
                        }
                    }
                    _ => {}
                }

                // Track Pac-Man's starting position
                if character == 'X' {
                    pacman_start = Some(IVec2::new(x as i32, y as i32));
                }

                tiles[x][y] = tile;
            }
        }

        // Validate house door configuration
        let house_door_count = house_door.iter().filter(|x| x.is_some()).count();
        if house_door_count != 2 {
            return Err(ParseError::InvalidHouseDoorCount(house_door_count));
        }

        Ok(ParsedMap {
            tiles,
            house_door,
            tunnel_ends,
            pacman_start,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RAW_BOARD;

    #[test]
    fn test_parse_character() {
        assert!(matches!(MapTileParser::parse_character('#').unwrap(), MapTile::Wall));
        assert!(matches!(MapTileParser::parse_character('.').unwrap(), MapTile::Pellet));
        assert!(matches!(MapTileParser::parse_character('o').unwrap(), MapTile::PowerPellet));
        assert!(matches!(MapTileParser::parse_character(' ').unwrap(), MapTile::Empty));
        assert!(matches!(MapTileParser::parse_character('T').unwrap(), MapTile::Tunnel));
        assert!(matches!(MapTileParser::parse_character('X').unwrap(), MapTile::Empty));
        assert!(matches!(MapTileParser::parse_character('=').unwrap(), MapTile::Wall));

        // Test invalid character
        assert!(MapTileParser::parse_character('Z').is_err());
    }

    #[test]
    fn test_parse_board() {
        let result = MapTileParser::parse_board(RAW_BOARD);
        assert!(result.is_ok());

        let parsed = result.unwrap();

        // Verify we have tiles
        assert_eq!(parsed.tiles.len(), BOARD_CELL_SIZE.x as usize);
        assert_eq!(parsed.tiles[0].len(), BOARD_CELL_SIZE.y as usize);

        // Verify we found house door positions
        assert!(parsed.house_door[0].is_some());
        assert!(parsed.house_door[1].is_some());

        // Verify we found tunnel ends
        assert!(parsed.tunnel_ends[0].is_some());
        assert!(parsed.tunnel_ends[1].is_some());

        // Verify we found Pac-Man's starting position
        assert!(parsed.pacman_start.is_some());
    }

    #[test]
    fn test_parse_board_invalid_character() {
        let mut invalid_board = RAW_BOARD.clone();
        invalid_board[0] = "###########################Z";

        let result = MapTileParser::parse_board(invalid_board);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnknownCharacter('Z')));
    }
}
