pub mod blinky;
pub mod ghost;
pub mod pacman;

use crate::{
    constants::{MapTile, BOARD_OFFSET, BOARD_WIDTH, CELL_SIZE},
    direction::Direction,
    map::Map,
    modulation::SimpleTickModulator,
};
use glam::{IVec2, UVec2};
use std::cell::RefCell;
use std::rc::Rc;

/// A trait for game objects that can be moved and rendered.
pub trait Entity {
    /// Returns a reference to the base entity (position, etc).
    fn base(&self) -> &StaticEntity;

    /// Returns true if the entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let a = self.base().pixel_position;
        let b = other.base().pixel_position;
        a == b
    }
}

/// A trait for entities that can move and interact with the map.
pub trait Moving {
    fn move_forward(&mut self);
    fn update_cell_position(&mut self);
    fn next_cell(&self, direction: Option<Direction>) -> IVec2;
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool;
    fn handle_tunnel(&mut self) -> bool;
    fn is_grid_aligned(&self) -> bool;
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool;
}

/// A struct for static (non-moving) entities with position only.
pub struct StaticEntity {
    pub pixel_position: IVec2,
    pub cell_position: UVec2,
}

impl StaticEntity {
    pub fn new(pixel_position: IVec2, cell_position: UVec2) -> Self {
        Self {
            pixel_position,
            cell_position,
        }
    }
}

/// A struct for movable game entities with position, direction, speed, and modulation.
pub struct MovableEntity {
    pub base: StaticEntity,
    pub direction: Direction,
    pub speed: u32,
    pub modulation: SimpleTickModulator,
    pub in_tunnel: bool,
    pub map: Rc<RefCell<Map>>,
}

impl MovableEntity {
    pub fn new(
        pixel_position: IVec2,
        cell_position: UVec2,
        direction: Direction,
        speed: u32,
        modulation: SimpleTickModulator,
        map: Rc<RefCell<Map>>,
    ) -> Self {
        Self {
            base: StaticEntity::new(pixel_position, cell_position),
            direction,
            speed,
            modulation,
            in_tunnel: false,
            map,
        }
    }

    /// Returns the position within the current cell, in pixels.
    pub fn internal_position(&self) -> UVec2 {
        UVec2::new(
            (self.base.pixel_position.x as u32) % CELL_SIZE,
            (self.base.pixel_position.y as u32) % CELL_SIZE,
        )
    }
}

impl Entity for MovableEntity {
    fn base(&self) -> &StaticEntity {
        &self.base
    }
}

impl Moving for MovableEntity {
    fn move_forward(&mut self) {
        let speed = self.speed as i32;
        match self.direction {
            Direction::Right => self.base.pixel_position.x += speed,
            Direction::Left => self.base.pixel_position.x -= speed,
            Direction::Up => self.base.pixel_position.y -= speed,
            Direction::Down => self.base.pixel_position.y += speed,
        }
    }
    fn update_cell_position(&mut self) {
        self.base.cell_position = UVec2::new(
            (self.base.pixel_position.x as u32 / CELL_SIZE) - BOARD_OFFSET.0,
            (self.base.pixel_position.y as u32 / CELL_SIZE) - BOARD_OFFSET.1,
        );
    }
    fn next_cell(&self, direction: Option<Direction>) -> IVec2 {
        let (x, y) = direction.unwrap_or(self.direction).offset();
        IVec2::new(self.base.cell_position.x as i32 + x, self.base.cell_position.y as i32 + y)
    }
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        let next_cell = self.next_cell(direction);
        matches!(self.map.borrow().get_tile(next_cell), Some(MapTile::Wall))
    }
    fn handle_tunnel(&mut self) -> bool {
        if !self.in_tunnel {
            let current_tile = self
                .map
                .borrow()
                .get_tile(IVec2::new(self.base.cell_position.x as i32, self.base.cell_position.y as i32));
            if matches!(current_tile, Some(MapTile::Tunnel)) {
                self.in_tunnel = true;
            }
        }
        if self.in_tunnel {
            if self.base.cell_position.x == 0 {
                self.base.cell_position.x = BOARD_WIDTH - 2;
                self.base.pixel_position = Map::cell_to_pixel(self.base.cell_position);
                self.in_tunnel = false;
                true
            } else if self.base.cell_position.x == BOARD_WIDTH - 1 {
                self.base.cell_position.x = 1;
                self.base.pixel_position = Map::cell_to_pixel(self.base.cell_position);
                self.in_tunnel = false;
                true
            } else {
                true
            }
        } else {
            false
        }
    }
    fn is_grid_aligned(&self) -> bool {
        self.internal_position() == UVec2::ZERO
    }
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
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

impl Entity for StaticEntity {
    fn base(&self) -> &StaticEntity {
        self
    }
}

/// A trait for entities that can be rendered to the screen.
pub trait Renderable {
    fn render(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>);
}
