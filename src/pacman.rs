use std::rc::Rc;

use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};
use tracing::event;

use crate::{
    animation::AnimatedTexture,
    constants::MapTile,
    constants::{CELL_SIZE, BOARD_OFFSET},
    direction::Direction,
    entity::Entity,
    map::Map,
    modulation::{SimpleTickModulator, TickModulator},
};

pub struct Pacman<'a> {
    // Absolute position on the board (precise)
    pub position: (i32, i32),
    pub direction: Direction,
    pub next_direction: Option<Direction>,
    pub stopped: bool,
    map: Rc<Map>,
    speed: u32,
    modulation: SimpleTickModulator,
    sprite: AnimatedTexture<'a>,
}

impl Pacman<'_> {
    pub fn new<'a>(starting_position: (u32, u32), atlas: Texture<'a>, map: Rc<Map>) -> Pacman<'a> {
        Pacman {
            position: Map::cell_to_pixel(starting_position),
            direction: Direction::Right,
            next_direction: None,
            speed: 2,
            map,
            stopped: false,
            modulation: SimpleTickModulator::new(0.9333),
            sprite: AnimatedTexture::new(atlas, 4, 3, 32, 32, Some((-4, -4))),
        }
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        // When stopped, render the last frame of the animation
        if self.stopped {
            self.sprite
                .render_until(canvas, self.position, self.direction, 2);
        } else {
            self.sprite.render(canvas, self.position, self.direction);
        }
    }

    pub fn next_cell(&self, direction: Option<Direction>) -> (i32, i32) {
        let (x, y) = direction.unwrap_or(self.direction).offset();
        let cell = self.cell_position();
        (cell.0 as i32 + x, cell.1 as i32 + y)
    }

    fn handle_requested_direction(&mut self) {
        if self.next_direction.is_none() { return; }
        if self.next_direction.unwrap() == self.direction { 
            self.next_direction = None;
            return;
        }

        let proposed_next_cell = self.next_cell(self.next_direction);
        let proposed_next_tile = self.map.get_tile(proposed_next_cell).unwrap_or(MapTile::Empty);
        if proposed_next_tile != MapTile::Wall {
            self.direction = self.next_direction.unwrap();
            self.next_direction = None;
        }
    }
}

impl Entity for Pacman<'_> {
    fn is_colliding(&self, other: &dyn Entity) -> bool {
        let (x, y) = self.position();
        let (other_x, other_y) = other.position();
        x == other_x && y == other_y
    }

    fn position(&self) -> (i32, i32) {
        self.position
    }

    fn cell_position(&self) -> (u32, u32) {
        let (x, y) = self.position;
        ((x as u32 / CELL_SIZE) - BOARD_OFFSET.0, (y as u32 / CELL_SIZE) - BOARD_OFFSET.1)
    }

    fn internal_position(&self) -> (u32, u32) {
        let (x, y) = self.position();
        (x as u32 % CELL_SIZE, y as u32 % CELL_SIZE)
    }

    fn tick(&mut self) {
        let can_change = self.internal_position() == (0, 0);

        if can_change {
            self.handle_requested_direction();

            let next = self.next_cell(None);
            let next_tile = self.map.get_tile(next).unwrap_or(MapTile::Empty);

            if !self.stopped && next_tile == MapTile::Wall {
                event!(tracing::Level::DEBUG, "Wall collision. Stopping.");
                self.stopped = true;
            } else if self.stopped && next_tile != MapTile::Wall {
                event!(tracing::Level::DEBUG, "Wall collision resolved. Moving.");
                self.stopped = false;
            }
        }

        if !self.stopped && self.modulation.next() {
            let speed = self.speed as i32;
            match self.direction {
                Direction::Right => {
                    self.position.0 += speed;
                }
                Direction::Left => {
                    self.position.0 -= speed;
                }
                Direction::Up => {
                    self.position.1 -= speed;
                }
                Direction::Down => {
                    self.position.1 += speed;
                }
            }
        }
    }
}
