use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::{animation::AnimatedTexture, direction::Direction, entity::Entity, constants::CELL_SIZE};

pub struct Pacman<'a> {
    // Absolute position on the board (precise)
    pub position: (i32, i32),
    pub direction: Direction,
    pub next_direction: Option<Direction>,
    speed: u32,
    sprite: AnimatedTexture<'a>,
}

impl Pacman<'_> {
    pub fn new<'a>(starting_position: Option<(i32, i32)>, atlas: Texture<'a>) -> Pacman<'a> {
        Pacman {
            position: starting_position.unwrap_or((0i32, 0i32)),
            direction: Direction::Right,
            next_direction: None,
            speed: 2,
            sprite: AnimatedTexture::new(atlas, 4, 3, 32, 32, Some((-4, -4))),
        }
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        self.sprite.render(canvas, self.position, self.direction);
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
        println!("{:?} ({:?})", self.internal_position(), can_change);
        if can_change {
            if let Some(direction) = self.next_direction {
                self.direction = direction;
                self.next_direction = None;
            }
        }
        
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
