//! A structured representation of all sprite assets in the game.
//!
//! This module provides a set of enums to represent every sprite, allowing for
//! type-safe access to asset paths and avoiding the use of raw strings.
//! The `GameSprite` enum is the main entry point, and its `to_path` method
//! generates the correct path for a given sprite in the texture atlas.

use crate::map::direction::Direction;
use crate::systems::components::Ghost;

/// Represents the different sprites for Pac-Man.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacmanSprite {
    /// A moving Pac-Man sprite for a given direction and animation frame.
    Moving(Direction, u8),
    /// The full, closed-mouth Pac-Man sprite.
    Full,
}

/// Represents the color of a frightened ghost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrightenedColor {
    Blue,
    White,
}

/// Represents the different sprites for ghosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostSprite {
    /// The normal appearance of a ghost for a given type, direction, and animation frame.
    Normal(Ghost, Direction, u8),
    /// The frightened appearance of a ghost, with a specific color and animation frame.
    Frightened(FrightenedColor, u8),
    /// The "eyes only" appearance of a ghost after being eaten.
    Eyes(Direction),
}

/// Represents the different sprites for the maze and collectibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MazeSprite {
    /// A specific tile of the maze.
    Tile(u8),
    /// A standard pellet.
    Pellet,
    /// An energizer/power pellet.
    Energizer,
}

/// A top-level enum that encompasses all game sprites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSprite {
    Pacman(PacmanSprite),
    Ghost(GhostSprite),
    Maze(MazeSprite),
}

impl GameSprite {
    /// Generates the asset path for the sprite.
    ///
    /// This path corresponds to the filename in the texture atlas JSON file.
    pub fn to_path(self) -> String {
        match self {
            GameSprite::Pacman(sprite) => match sprite {
                PacmanSprite::Moving(dir, frame) => {
                    let frame_char = match frame {
                        0 => 'a',
                        1 => 'b',
                        _ => panic!("Invalid animation frame"),
                    };
                    format!("pacman/{}_{}.png", dir.as_ref().to_lowercase(), frame_char)
                }
                PacmanSprite::Full => "pacman/full.png".to_string(),
            },
            GameSprite::Ghost(sprite) => match sprite {
                GhostSprite::Normal(ghost, dir, frame) => {
                    let frame_char = match frame {
                        0 => 'a',
                        1 => 'b',
                        _ => panic!("Invalid animation frame"),
                    };
                    format!("ghost/{}/{}_{}.png", ghost.as_str(), dir.as_ref().to_lowercase(), frame_char)
                }
                GhostSprite::Frightened(color, frame) => {
                    let frame_char = match frame {
                        0 => 'a',
                        1 => 'b',
                        _ => panic!("Invalid animation frame"),
                    };
                    let color_str = match color {
                        FrightenedColor::Blue => "blue",
                        FrightenedColor::White => "white",
                    };
                    format!("ghost/frightened/{}_{}.png", color_str, frame_char)
                }
                GhostSprite::Eyes(dir) => format!("ghost/eyes/{}.png", dir.as_ref().to_lowercase()),
            },
            GameSprite::Maze(sprite) => match sprite {
                MazeSprite::Tile(index) => format!("maze/tiles/{}.png", index),
                MazeSprite::Pellet => "maze/pellet.png".to_string(),
                MazeSprite::Energizer => "maze/energizer.png".to_string(),
            },
        }
    }
}
