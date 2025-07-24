//! This module defines the `Direction` enum, which is used to represent the
//! direction of an entity.
use glam::IVec2;
use sdl2::keyboard::Keycode;

/// An enum representing the direction of an entity.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns the angle of the direction in degrees.
    pub fn angle(&self) -> f64 {
        match self {
            Direction::Right => 0f64,
            Direction::Down => 90f64,
            Direction::Left => 180f64,
            Direction::Up => 270f64,
        }
    }

    /// Returns the offset of the direction as a tuple of (x, y).
    pub fn offset(&self) -> IVec2 {
        match self {
            Direction::Right => IVec2::new(1, 0),
            Direction::Down => IVec2::new(0, 1),
            Direction::Left => IVec2::new(-1, 0),
            Direction::Up => IVec2::new(0, -1),
        }
    }

    /// Returns the opposite direction.
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
        }
    }

    /// Creates a `Direction` from a `Keycode`.
    ///
    /// # Arguments
    ///
    /// * `keycode` - The keycode to convert.
    pub fn from_keycode(keycode: Keycode) -> Option<Direction> {
        match keycode {
            Keycode::D | Keycode::Right => Some(Direction::Right),
            Keycode::A | Keycode::Left => Some(Direction::Left),
            Keycode::W | Keycode::Up => Some(Direction::Up),
            Keycode::S | Keycode::Down => Some(Direction::Down),
            _ => None,
        }
    }
}
