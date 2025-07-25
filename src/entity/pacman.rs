//! This module defines the Pac-Man entity, including its behavior and rendering.
use std::cell::RefCell;
use std::rc::Rc;

use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::{
    entity::{direction::Direction, Entity, MovableEntity, Moving, Renderable, StaticEntity},
    map::Map,
    modulation::{SimpleTickModulator, TickModulator},
    texture::animated::AnimatedAtlasTexture,
    texture::FrameDrawn,
};

use glam::{IVec2, UVec2};

/// The Pac-Man entity.
pub struct Pacman {
    /// Shared movement and position fields.
    pub base: MovableEntity,
    /// The next direction of Pac-Man, which will be applied when Pac-Man is next aligned with the grid.
    pub next_direction: Option<Direction>,
    /// Whether Pac-Man is currently stopped.
    pub stopped: bool,
    pub sprite: AnimatedAtlasTexture,
}

impl Entity for Pacman {
    fn base(&self) -> &StaticEntity {
        &self.base.base
    }
}

impl Moving for Pacman {
    fn move_forward(&mut self) {
        self.base.move_forward();
    }
    fn update_cell_position(&mut self) {
        self.base.update_cell_position();
    }
    fn next_cell(&self, direction: Option<Direction>) -> IVec2 {
        self.base.next_cell(direction)
    }
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        self.base.is_wall_ahead(direction)
    }
    fn handle_tunnel(&mut self) -> bool {
        self.base.handle_tunnel()
    }
    fn is_grid_aligned(&self) -> bool {
        self.base.is_grid_aligned()
    }
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
        self.base.set_direction_if_valid(new_direction)
    }
}

impl Pacman {
    /// Creates a new `Pacman` instance.
    pub fn new(starting_position: UVec2, atlas: Texture<'_>, map: Rc<RefCell<Map>>) -> Pacman {
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
            sprite: AnimatedAtlasTexture::new(
                unsafe { crate::texture::atlas::texture_to_static(atlas) },
                2,
                3,
                32,
                32,
                Some(IVec2::new(-4, -4)),
            ),
        }
    }

    /// Handles a requested direction change.
    fn handle_direction_change(&mut self) -> bool {
        match self.next_direction {
            None => return false,
            Some(next_direction) => {
                if <Pacman as Moving>::set_direction_if_valid(self, next_direction) {
                    self.next_direction = None;
                    return true;
                }
            }
        }
        false
    }

    /// Returns the internal position of Pac-Man, rounded down to the nearest even number.
    fn internal_position_even(&self) -> UVec2 {
        let pos = self.base.internal_position();
        UVec2::new((pos.x / 2) * 2, (pos.y / 2) * 2)
    }

    pub fn tick(&mut self) {
        let can_change = self.internal_position_even() == UVec2::ZERO;
        if can_change {
            <Pacman as Moving>::update_cell_position(self);
            if !<Pacman as Moving>::handle_tunnel(self) {
                self.handle_direction_change();
                if !self.stopped && <Pacman as Moving>::is_wall_ahead(self, None) {
                    self.stopped = true;
                } else if self.stopped && !<Pacman as Moving>::is_wall_ahead(self, None) {
                    self.stopped = false;
                }
            }
        }
        if !self.stopped && self.base.modulation.next() {
            <Pacman as Moving>::move_forward(self);
            if self.internal_position_even() == UVec2::ZERO {
                <Pacman as Moving>::update_cell_position(self);
            }
        }
    }
}

impl Renderable for Pacman {
    fn render(&self, canvas: &mut Canvas<Window>) {
        let pos = self.base.base.pixel_position;
        let dir = self.base.direction;
        if self.stopped {
            self.sprite.render(canvas, pos, dir, Some(2));
        } else {
            self.sprite.render(canvas, pos, dir, None);
        }
    }
}
