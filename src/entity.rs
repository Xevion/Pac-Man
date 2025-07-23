use crate::{
    constants::{MapTile, BOARD_OFFSET, BOARD_WIDTH, CELL_SIZE},
    direction::Direction,
    map::Map,
    modulation::SimpleTickModulator,
};
use std::cell::RefCell;
use std::rc::Rc;

/// A trait for game objects that can be moved and rendered.
pub trait Entity {
    /// Returns a reference to the base MovableEntity.
    fn base(&self) -> &MovableEntity;

    /// Returns true if the entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.base().pixel_position;
        let (other_x, other_y) = other.base().pixel_position;
        x == other_x && y == other_y
    }

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
    pub direction: Direction,
    /// Movement speed (pixels per tick).
    pub speed: u32,
    /// Movement modulator for controlling speed.
    pub modulation: SimpleTickModulator,
    /// Whether the entity is currently in a tunnel.
    pub in_tunnel: bool,
    /// Reference to the game map.
    pub map: Rc<RefCell<Map>>,
}

impl MovableEntity {
    /// Creates a new MovableEntity.
    pub fn new(
        pixel_position: (i32, i32),
        cell_position: (u32, u32),
        direction: Direction,
        speed: u32,
        modulation: SimpleTickModulator,
        map: Rc<RefCell<Map>>,
    ) -> Self {
        Self {
            pixel_position,
            cell_position,
            direction,
            speed,
            modulation,
            in_tunnel: false,
            map,
        }
    }

    /// Returns the position within the current cell, in pixels.
    pub fn internal_position(&self) -> (u32, u32) {
        (
            self.pixel_position.0 as u32 % CELL_SIZE,
            self.pixel_position.1 as u32 % CELL_SIZE,
        )
    }

    /// Move the entity in its current direction by its speed.
    pub fn move_forward(&mut self) {
        let speed = self.speed as i32;
        match self.direction {
            Direction::Right => self.pixel_position.0 += speed,
            Direction::Left => self.pixel_position.0 -= speed,
            Direction::Up => self.pixel_position.1 -= speed,
            Direction::Down => self.pixel_position.1 += speed,
        }
    }

    /// Updates the cell position based on the current pixel position.
    pub fn update_cell_position(&mut self) {
        self.cell_position = (
            (self.pixel_position.0 as u32 / CELL_SIZE) - BOARD_OFFSET.0,
            (self.pixel_position.1 as u32 / CELL_SIZE) - BOARD_OFFSET.1,
        );
    }

    /// Calculates the next cell in the given direction.
    pub fn next_cell(&self, direction: Option<Direction>) -> (i32, i32) {
        let (x, y) = direction.unwrap_or(self.direction).offset();
        (
            self.cell_position.0 as i32 + x,
            self.cell_position.1 as i32 + y,
        )
    }

    /// Returns true if the next cell in the given direction is a wall.
    pub fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        let next_cell = self.next_cell(direction);
        matches!(self.map.borrow().get_tile(next_cell), Some(MapTile::Wall))
    }

    /// Handles tunnel movement and wrapping.
    /// Returns true if the entity is in a tunnel and was handled.
    pub fn handle_tunnel(&mut self) -> bool {
        if !self.in_tunnel {
            let current_tile = self
                .map
                .borrow()
                .get_tile((self.cell_position.0 as i32, self.cell_position.1 as i32));
            if matches!(current_tile, Some(MapTile::Tunnel)) {
                self.in_tunnel = true;
            }
        }

        if self.in_tunnel {
            // If out of bounds, teleport to the opposite side and exit tunnel
            if self.cell_position.0 == 0 {
                self.cell_position.0 = BOARD_WIDTH - 2;
                self.pixel_position =
                    Map::cell_to_pixel((self.cell_position.0, self.cell_position.1));
                self.in_tunnel = false;
                true
            } else if self.cell_position.0 == BOARD_WIDTH - 1 {
                self.cell_position.0 = 1;
                self.pixel_position =
                    Map::cell_to_pixel((self.cell_position.0, self.cell_position.1));
                self.in_tunnel = false;
                true
            } else {
                // Still in tunnel, keep moving
                true
            }
        } else {
            false
        }
    }

    /// Returns true if the entity is aligned with the grid.
    pub fn is_grid_aligned(&self) -> bool {
        self.internal_position() == (0, 0)
    }

    /// Attempts to set the direction if the next cell is not a wall.
    /// Returns true if the direction was changed.
    pub fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
        if new_direction == self.direction {
            return false;
        }
        if self.is_wall_ahead(Some(new_direction)) {
            return false;
        }
        self.direction = new_direction;
        true
    }
}

/// A trait for entities that can be rendered to the screen.
pub trait Renderable {
    /// Renders the entity to the canvas.
    fn render(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>);
}
