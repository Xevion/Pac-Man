use crate::map::direction::Direction;
use crate::texture::sprite::AtlasTile;

/// Fixed-size tile sequence that avoids heap allocation
#[derive(Clone, Copy, Debug)]
pub struct TileSequence {
    tiles: [AtlasTile; 4], // Fixed array, max 4 frames
    count: usize,          // Actual number of frames used
}

impl TileSequence {
    /// Creates a new tile sequence from a slice of tiles
    pub fn new(tiles: &[AtlasTile]) -> Self {
        let mut tile_array = [AtlasTile {
            pos: glam::U16Vec2::ZERO,
            size: glam::U16Vec2::ZERO,
            color: None,
        }; 4];

        let count = tiles.len().min(4);
        tile_array[..count].copy_from_slice(&tiles[..count]);

        Self {
            tiles: tile_array,
            count,
        }
    }

    /// Returns the tile at the given frame index, wrapping if necessary
    pub fn get_tile(&self, frame: usize) -> AtlasTile {
        if self.count == 0 {
            // Return a default empty tile if no tiles
            AtlasTile {
                pos: glam::U16Vec2::ZERO,
                size: glam::U16Vec2::ZERO,
                color: None,
            }
        } else {
            self.tiles[frame % self.count]
        }
    }

    /// Returns true if this sequence has no tiles
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Type-safe directional tile storage with named fields
#[derive(Clone, Copy, Debug)]
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
