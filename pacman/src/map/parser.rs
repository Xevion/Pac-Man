//! Map parsing functionality for converting raw board layouts into structured data.

use crate::constants::{MapTile, BOARD_CELL_SIZE};
use crate::error::ParseError;
use glam::I8Vec2;

/// Structured representation of parsed ASCII board layout with extracted special positions.
///
/// Contains the complete board state after character-to-tile conversion, along with
/// the locations of special gameplay elements that require additional processing
/// during graph construction. Special positions are extracted during parsing to
/// enable proper map builder initialization.
#[derive(Debug)]
pub struct ParsedMap {
    /// 2D array of tiles converted from ASCII characters
    pub tiles: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// Two positions marking the ghost house entrance (represented by '=' characters)
    pub house_door: [Option<I8Vec2>; 2],
    /// Two positions marking tunnel portals for wraparound teleportation ('T' characters)
    pub tunnel_ends: [Option<I8Vec2>; 2],
    /// Starting position for Pac-Man (marked by 'X' character in the layout)
    pub pacman_start: Option<I8Vec2>,
}

/// Parser for converting raw board layouts into structured map data.
pub struct MapTileParser;

impl MapTileParser {
    /// Converts ASCII characters from the board layout into corresponding tile types.
    ///
    /// Interprets the character-based maze representation: walls (`#`), collectible
    /// pellets (`.` and `o`), traversable spaces (` `), tunnel entrances (`T`),
    /// ghost house doors (`=`), and entity spawn markers (`X`). Special characters
    /// that don't represent tiles in the final map (like spawn markers) are
    /// converted to `Empty` tiles while their positions are tracked separately.
    ///
    /// # Errors
    ///
    /// Returns `ParseError::UnknownCharacter` for any character not defined
    /// in the game's ASCII art vocabulary.
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
        // Validate board dimensions
        if raw_board.len() != BOARD_CELL_SIZE.y as usize {
            return Err(ParseError::ParseFailed(format!(
                "Invalid board height: expected {}, got {}",
                BOARD_CELL_SIZE.y,
                raw_board.len()
            )));
        }

        for (i, line) in raw_board.iter().enumerate() {
            if line.len() != BOARD_CELL_SIZE.x as usize {
                return Err(ParseError::ParseFailed(format!(
                    "Invalid board width at line {}: expected {}, got {}",
                    i,
                    BOARD_CELL_SIZE.x,
                    line.len()
                )));
            }
        }
        let mut tiles = [[MapTile::Empty; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize];
        let mut house_door = [None; 2];
        let mut tunnel_ends = [None; 2];
        let mut pacman_start: Option<I8Vec2> = None;

        for (y, line) in raw_board.iter().enumerate().take(BOARD_CELL_SIZE.y as usize) {
            for (x, character) in line.chars().enumerate().take(BOARD_CELL_SIZE.x as usize) {
                let tile = Self::parse_character(character)?;

                // Track special positions
                match tile {
                    MapTile::Tunnel => {
                        if tunnel_ends[0].is_none() {
                            tunnel_ends[0] = Some(I8Vec2::new(x as i8, y as i8));
                        } else {
                            tunnel_ends[1] = Some(I8Vec2::new(x as i8, y as i8));
                        }
                    }
                    MapTile::Wall if character == '=' => {
                        if house_door[0].is_none() {
                            house_door[0] = Some(I8Vec2::new(x as i8, y as i8));
                        } else {
                            house_door[1] = Some(I8Vec2::new(x as i8, y as i8));
                        }
                    }
                    _ => {}
                }

                // Track Pac-Man's starting position
                if character == 'X' {
                    pacman_start = Some(I8Vec2::new(x as i8, y as i8));
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
