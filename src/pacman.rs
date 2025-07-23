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
    constants::MapTile,
    constants::{BOARD_OFFSET, BOARD_WIDTH, CELL_SIZE},
    direction::Direction,
    entity::{Entity, MovableEntity},
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
    map: Rc<RefCell<Map>>,
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
            ),
            next_direction: None,
            stopped: false,
            map,
            sprite: AnimatedTexture::new(atlas, 2, 3, 32, 32, Some((-4, -4))),
        }
    }

    /// Renders Pac-Man to the canvas.
    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
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

    /// Calculates the next cell in the given direction.
    pub fn next_cell(&self, direction: Option<Direction>) -> (i32, i32) {
        let (x, y) = direction.unwrap_or(self.base.direction).offset();
        let cell = self.base.cell_position;
        (cell.0 as i32 + x, cell.1 as i32 + y)
    }

    /// Handles a requested direction change.
    ///
    /// The direction change is only applied if the next tile in the requested
    /// direction is not a wall.
    fn handle_direction_change(&mut self) -> bool {
        match self.next_direction {
            // If there is no next direction, do nothing.
            None => return false,
            // If the next direction is the same as the current direction, do nothing.
            Some(next_direction) => {
                if next_direction == self.base.direction {
                    self.next_direction = None;
                    return false;
                }
            }
        }

        // Get the next cell in the proposed direction.
        let proposed_next_cell = self.next_cell(self.next_direction);
        let proposed_next_tile = self
            .map
            .borrow()
            .get_tile(proposed_next_cell)
            .unwrap_or(MapTile::Empty);

        // If the next tile is a wall, do nothing.
        if proposed_next_tile == MapTile::Wall {
            return false;
        }

        // If the next tile is not a wall, change direction.
        event!(
            tracing::Level::DEBUG,
            "Direction change: {:?} -> {:?} at position ({}, {}) internal ({}, {})",
            self.base.direction,
            self.next_direction.unwrap(),
            self.base.pixel_position.0,
            self.base.pixel_position.1,
            self.base.internal_position().0,
            self.base.internal_position().1
        );
        self.base.direction = self.next_direction.unwrap();
        self.next_direction = None;

        true
    }

    /// Returns the internal position of Pac-Man, rounded down to the nearest
    /// even number.
    ///
    /// This is used to ensure that Pac-Man is aligned with the grid before
    /// changing direction.
    fn internal_position_even(&self) -> (u32, u32) {
        let (x, y) = self.base.internal_position();
        ((x / 2u32) * 2u32, (y / 2u32) * 2u32)
    }
}

impl Entity for Pacman<'_> {
    fn base(&self) -> &MovableEntity {
        &self.base
    }

    /// Returns true if the Pac-Man entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.base.pixel_position;
        let (other_x, other_y) = other.base().pixel_position;
        x == other_x && y == other_y
    }

    /// Ticks the Pac-Man entity.
    fn tick(&mut self) {
        // Pac-Man can only change direction when he is perfectly aligned with the grid.
        let can_change = self.internal_position_even() == (0, 0);

        if can_change {
            self.base.cell_position = (
                (self.base.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.base.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );

            let current_tile = self
                .map
                .borrow()
                .get_tile((
                    self.base.cell_position.0 as i32,
                    self.base.cell_position.1 as i32,
                ))
                .unwrap_or(MapTile::Empty);
            if current_tile == MapTile::Tunnel {
                self.base.in_tunnel = true;
            }

            // Tunnel logic: if in tunnel, force movement and prevent direction change
            if self.base.in_tunnel {
                // If out of bounds, teleport to the opposite side and exit tunnel
                if self.base.cell_position.0 == 0 {
                    self.base.cell_position.0 = BOARD_WIDTH - 2;
                    self.base.pixel_position = Map::cell_to_pixel((
                        self.base.cell_position.0 + 1,
                        self.base.cell_position.1,
                    ));
                    self.base.in_tunnel = false;
                } else if self.base.cell_position.0 == BOARD_WIDTH - 1 {
                    self.base.cell_position.0 = 1;
                    self.base.pixel_position = Map::cell_to_pixel((
                        self.base.cell_position.0 - 1,
                        self.base.cell_position.1,
                    ));
                    self.base.in_tunnel = false;
                } else {
                    // While in tunnel, do not allow direction change
                    // and always move in the current direction
                }
            } else {
                // Handle direction change as normal
                self.handle_direction_change();

                // Check if the next tile in the current direction is a wall.
                let next_tile_position = self.next_cell(None);
                let next_tile = self
                    .map
                    .borrow()
                    .get_tile(next_tile_position)
                    .unwrap_or(MapTile::Empty);

                if !self.stopped && next_tile == MapTile::Wall {
                    self.stopped = true;
                } else if self.stopped && next_tile != MapTile::Wall {
                    self.stopped = false;
                }
            }
        }

        if !self.stopped {
            if self.base.modulation.next() {
                self.base.move_forward();
                // Update the cell position if Pac-Man is aligned with the grid.
                if self.internal_position_even() == (0, 0) {
                    self.base.cell_position = (
                        (self.base.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                        (self.base.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
                    );
                }
            }
        }
    }
}
