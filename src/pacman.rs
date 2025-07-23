//! This module defines the Pac-Man entity, including its behavior and rendering.
use std::cell::RefCell;
use std::rc::Rc;

use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};
use tracing::event;

use crate::{
    animation::AnimatedTexture,
    direction::Direction,
    entity::{Entity, MovableEntity, Renderable},
    map::Map,
    modulation::{SimpleTickModulator, TickModulator},
};

/// The Pac-Man entity.
pub struct Pacman<'a> {
    /// Shared movement and position fields.
    pub base: MovableEntity,
    /// The next direction of Pac-Man, which will be applied when Pac-Man is next aligned with the grid.
    pub next_direction: Option<Direction>,
    /// Whether Pac-Man is currently stopped.
    pub stopped: bool,
    sprite: AnimatedTexture<'a>,
}

impl Pacman<'_> {
    /// Creates a new `Pacman` instance.
    pub fn new<'a>(
        starting_position: (u32, u32),
        atlas: Texture<'a>,
        map: Rc<RefCell<Map>>,
    ) -> Pacman<'a> {
        let pixel_position = Map::cell_to_pixel(starting_position);
        Pacman {
            base: MovableEntity::new(
                pixel_position,
                starting_position,
                Direction::Right,
                3,
                SimpleTickModulator::new(1.0),
                map,
            ),
            next_direction: None,
            stopped: false,
            sprite: AnimatedTexture::new(atlas, 2, 3, 32, 32, Some((-4, -4))),
        }
    }

    /// Handles a requested direction change.
    fn handle_direction_change(&mut self) -> bool {
        match self.next_direction {
            None => return false,
            Some(next_direction) => {
                if self.base.set_direction_if_valid(next_direction) {
                    self.next_direction = None;
                    return true;
                }
            }
        }
        false
    }

    /// Returns the internal position of Pac-Man, rounded down to the nearest even number.
    fn internal_position_even(&self) -> (u32, u32) {
        let (x, y) = self.base.internal_position();
        ((x / 2u32) * 2u32, (y / 2u32) * 2u32)
    }
}

impl Entity for Pacman<'_> {
    fn base(&self) -> &MovableEntity {
        &self.base
    }

    fn tick(&mut self) {
        let can_change = self.internal_position_even() == (0, 0);

        if can_change {
            self.base.update_cell_position();

            if !self.base.handle_tunnel() {
                // Handle direction change as normal if not in tunnel
                self.handle_direction_change();

                // Check if the next tile in the current direction is a wall
                if !self.stopped && self.base.is_wall_ahead(None) {
                    self.stopped = true;
                } else if self.stopped && !self.base.is_wall_ahead(None) {
                    self.stopped = false;
                }
            }
        }

        if !self.stopped && self.base.modulation.next() {
            self.base.move_forward();
            if self.internal_position_even() == (0, 0) {
                self.base.update_cell_position();
            }
        }
    }
}

impl Renderable for Pacman<'_> {
    fn render(&mut self, canvas: &mut Canvas<Window>) {
        if self.stopped {
            self.sprite.render_static(
                canvas,
                self.base.pixel_position,
                self.base.direction,
                Some(2),
            );
        } else {
            self.sprite
                .render(canvas, self.base.pixel_position, self.base.direction);
        }
    }
}
