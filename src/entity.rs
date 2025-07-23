//! This module defines the `Entity` trait, which is implemented by all game
//! objects that can be moved and rendered.

/// A trait for game objects that can be moved and rendered.
pub trait Entity {
    /// Returns a reference to the base MovableEntity.
    fn base(&self) -> &MovableEntity;

    /// Returns true if the entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool;

    /// Ticks the entity, which updates its state and position.
    fn tick(&mut self);
}

/// A struct for movable game entities with position, direction, speed, and modulation.
pub struct MovableEntity {
    /// The absolute position of the entity on the board, in pixels.
    pub pixel_position: (i32, i32),
    /// The position of the entity on the board, in grid coordinates.
    pub cell_position: (u32, u32),
    /// The current direction of the entity.
    pub direction: crate::direction::Direction,
    /// Movement speed (pixels per tick).
    pub speed: u32,
    /// Movement modulator for controlling speed.
    pub modulation: crate::modulation::SimpleTickModulator,
    /// Whether the entity is currently in a tunnel.
    pub in_tunnel: bool,
}

impl MovableEntity {
    /// Creates a new MovableEntity.
    pub fn new(
        pixel_position: (i32, i32),
        cell_position: (u32, u32),
        direction: crate::direction::Direction,
        speed: u32,
        modulation: crate::modulation::SimpleTickModulator,
    ) -> Self {
        Self {
            pixel_position,
            cell_position,
            direction,
            speed,
            modulation,
            in_tunnel: false,
        }
    }

    /// Returns the position within the current cell, in pixels.
    pub fn internal_position(&self) -> (u32, u32) {
        (
            self.pixel_position.0 as u32 % crate::constants::CELL_SIZE,
            self.pixel_position.1 as u32 % crate::constants::CELL_SIZE,
        )
    }

    /// Move the entity in its current direction by its speed.
    pub fn move_forward(&mut self) {
        let speed = self.speed as i32;
        use crate::direction::Direction;
        match self.direction {
            Direction::Right => self.pixel_position.0 += speed,
            Direction::Left => self.pixel_position.0 -= speed,
            Direction::Up => self.pixel_position.1 -= speed,
            Direction::Down => self.pixel_position.1 += speed,
        }
    }
}
