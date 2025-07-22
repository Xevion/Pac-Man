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
    entity::Entity,
    map::Map,
    modulation::{SimpleTickModulator, TickModulator},
};

/// The Pac-Man entity.
pub struct Pacman<'a> {
    /// The absolute position of Pac-Man on the board, in pixels.
    pub pixel_position: (i32, i32),
    /// The position of Pac-Man on the board, in grid coordinates.
    /// This is only updated at the moment Pac-Man is aligned with the grid.
    pub cell_position: (u32, u32),
    /// The current direction of Pac-Man.
    pub direction: Direction,
    /// The next direction of Pac-Man, which will be applied when Pac-Man is next aligned with the grid.
    pub next_direction: Option<Direction>,
    /// Whether Pac-Man is currently stopped.
    pub stopped: bool,
    map: Rc<RefCell<Map>>,
    speed: u32,
    modulation: SimpleTickModulator,
    sprite: AnimatedTexture<'a>,
    pub in_tunnel: bool,
}

impl Pacman<'_> {
    /// Creates a new `Pacman` instance.
    ///
    /// # Arguments
    ///
    /// * `starting_position` - The starting position of Pac-Man, in grid coordinates.
    /// * `atlas` - The texture atlas containing the Pac-Man sprites.
    /// * `map` - A reference to the game map.
    pub fn new<'a>(
        starting_position: (u32, u32),
        atlas: Texture<'a>,
        map: Rc<RefCell<Map>>,
    ) -> Pacman<'a> {
        Pacman {
            pixel_position: Map::cell_to_pixel(starting_position),
            cell_position: starting_position,
            direction: Direction::Right,
            next_direction: None,
            speed: 3,
            map,
            stopped: false,
            modulation: SimpleTickModulator::new(1.0),
            sprite: AnimatedTexture::new(atlas, 2, 3, 32, 32, Some((-4, -4))),
            in_tunnel: false,
        }
    }

    /// Renders Pac-Man to the canvas.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL canvas to render to.
    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        if self.stopped {
            self.sprite
                .render_static(canvas, self.pixel_position, self.direction, Some(2));
        } else {
            self.sprite
                .render(canvas, self.pixel_position, self.direction);
        }
    }

    /// Calculates the next cell in the given direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to check. If `None`, the current direction is used.
    pub fn next_cell(&self, direction: Option<Direction>) -> (i32, i32) {
        let (x, y) = direction.unwrap_or(self.direction).offset();
        let cell = self.cell_position;
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
                if next_direction == self.direction {
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
            self.direction,
            self.next_direction.unwrap(),
            self.pixel_position.0,
            self.pixel_position.1,
            self.internal_position().0,
            self.internal_position().1
        );
        self.direction = self.next_direction.unwrap();
        self.next_direction = None;

        true
    }

    /// Returns the internal position of Pac-Man, rounded down to the nearest
    /// even number.
    ///
    /// This is used to ensure that Pac-Man is aligned with the grid before
    /// changing direction.
    fn internal_position_even(&self) -> (u32, u32) {
        let (x, y) = self.internal_position();
        ((x / 2u32) * 2u32, (y / 2u32) * 2u32)
    }
}

impl Entity for Pacman<'_> {
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.position();
        let (other_x, other_y) = other.position();
        x == other_x && y == other_y
    }

    fn position(&self) -> (i32, i32) {
        self.pixel_position
    }

    fn cell_position(&self) -> (u32, u32) {
        self.cell_position
    }

    fn internal_position(&self) -> (u32, u32) {
        let (x, y) = self.position();
        (x as u32 % CELL_SIZE, y as u32 % CELL_SIZE)
    }

    fn tick(&mut self) {
        // Pac-Man can only change direction when he is perfectly aligned with the grid.
        let can_change = self.internal_position_even() == (0, 0);

        if can_change {
            self.cell_position = (
                (self.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                (self.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
            );

            let current_tile = self
                .map
                .borrow()
                .get_tile((self.cell_position.0 as i32, self.cell_position.1 as i32))
                .unwrap_or(MapTile::Empty);
            if current_tile == MapTile::Tunnel {
                self.in_tunnel = true;
            }

            // Tunnel logic: if in tunnel, force movement and prevent direction change
            if self.in_tunnel {
                // If out of bounds, teleport to the opposite side and exit tunnel
                if self.cell_position.0 == 0 {
                    self.cell_position.0 = BOARD_WIDTH - 2;
                    self.pixel_position =
                        Map::cell_to_pixel((self.cell_position.0 + 1, self.cell_position.1));
                    self.in_tunnel = false;
                } else if self.cell_position.0 == BOARD_WIDTH - 1 {
                    self.cell_position.0 = 1;
                    self.pixel_position =
                        Map::cell_to_pixel((self.cell_position.0 - 1, self.cell_position.1));
                    self.in_tunnel = false;
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
            if self.modulation.next() {
                let speed = self.speed as i32;
                match self.direction {
                    Direction::Right => {
                        self.pixel_position.0 += speed;
                    }
                    Direction::Left => {
                        self.pixel_position.0 -= speed;
                    }
                    Direction::Up => {
                        self.pixel_position.1 -= speed;
                    }
                    Direction::Down => {
                        self.pixel_position.1 += speed;
                    }
                }

                // Update the cell position if Pac-Man is aligned with the grid.
                if self.internal_position_even() == (0, 0) {
                    self.cell_position = (
                        (self.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
                        (self.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
                    );
                }
            }
        }
    }
}
