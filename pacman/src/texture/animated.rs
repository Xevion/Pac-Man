use glam::U16Vec2;

use crate::{map::direction::Direction, texture::sprite::AtlasTile};

/// A sequence of tiles for animation, backed by a vector.
#[derive(Debug, Clone)]
pub struct TileSequence {
    tiles: Vec<AtlasTile>,
}

impl TileSequence {
    /// Creates a new tile sequence from a slice of tiles.
    pub fn new(tiles: &[AtlasTile]) -> Self {
        Self { tiles: tiles.to_vec() }
    }

    /// Returns the tile at the given frame index, wrapping if necessary
    pub fn get_tile(&self, frame: usize) -> AtlasTile {
        if self.tiles.is_empty() {
            // Return a default or handle the error appropriately
            // For now, let's return a default tile, assuming it's a sensible default
            return AtlasTile {
                pos: U16Vec2::ZERO,
                size: U16Vec2::ZERO,
                color: None,
            };
        }
        self.tiles[frame % self.tiles.len()]
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Checks if the sequence contains any tiles.
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

/// A collection of tile sequences for each cardinal direction.
#[derive(Debug, Clone)]
pub struct DirectionalTiles {
    pub up: TileSequence,
    pub down: TileSequence,
    pub left: TileSequence,
    pub right: TileSequence,
}

impl DirectionalTiles {
    /// Creates a new DirectionalTiles with different sequences per direction
    pub fn new(up: TileSequence, down: TileSequence, left: TileSequence, right: TileSequence) -> Self {
        Self { up, down, left, right }
    }

    /// Gets the tile sequence for the given direction
    pub fn get(&self, direction: Direction) -> &TileSequence {
        match direction {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}
