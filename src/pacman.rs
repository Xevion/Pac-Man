use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::{
    constants::{BOARD, MapTile},
    animation::AnimatedTexture, constants::CELL_SIZE, direction::Direction, entity::Entity,
    modulation::SpeedModulator,
};

pub struct Pacman<'a> {
    // Absolute position on the board (precise)
    pub position: (i32, i32),
    pub direction: Direction,
    pub next_direction: Option<Direction>,
    pub stopped: bool,
    speed: u32,
    modulation: SpeedModulator,
    sprite: AnimatedTexture<'a>,
}

impl Pacman<'_> {
    pub fn new<'a>(starting_position: Option<(i32, i32)>, atlas: Texture<'a>) -> Pacman<'a> {
        Pacman {
            position: starting_position.unwrap_or((0i32, 0i32)),
            direction: Direction::Right,
            next_direction: None,
            speed: 2,
            stopped: false,
            modulation: SpeedModulator::new(0.9333),
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

    fn next_cell(&self) -> (i32, i32) {
        let (x, y) = self.direction.offset();
        let cell = self.cell_position();
        (cell.0 as i32 + x, cell.1 as i32 + y)
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
        let (x, y) = self.position();
        (x as u32 / CELL_SIZE, y as u32 / CELL_SIZE)
    }

    fn internal_position(&self) -> (u32, u32) {
        let (x, y) = self.position();
        (x as u32 % CELL_SIZE, y as u32 % CELL_SIZE)
    }

    fn tick(&mut self) {
        let can_change = self.internal_position() == (0, 0);
        if can_change {
            if let Some(direction) = self.next_direction {
                self.direction = direction;
                self.next_direction = None;
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

        let next = self.next_cell();
        if BOARD[next.1 as usize][next.0 as usize] == MapTile::Wall {
            self.stopped = true;
        }
    }
}
