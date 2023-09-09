use sdl2::{render::{Canvas, Texture}, video::Window};

use crate::{direction::Direction, entity::Entity, animation::AnimatedTexture};

pub struct Pacman<'a> {
    // Absolute position on the board (precise)
    pub position: (i32, i32),
    pub direction: Direction,
    sprite: AnimatedTexture<'a>,
}

impl Pacman<'_> {
    pub fn new<'a>(starting_position: Option<(i32, i32)>, atlas: Texture<'a>) -> Pacman<'a> {
        Pacman {
            position: starting_position.unwrap_or((0i32, 0i32)),
            direction: Direction::Right,
            sprite: AnimatedTexture::new(atlas, 4, 3, 32,32),
        }
    }

    pub fn render(&mut self,  canvas: &mut Canvas<Window>) {
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
        (x as u32 / 24, y as u32 / 24)
    }

    fn tick(&mut self) {
        match self.direction {
            Direction::Right => {
                self.position.0 += 1;
            }
            Direction::Left => {
                self.position.0 -= 1;
            }
            Direction::Up => {
                self.position.1 -= 1;
            }
            Direction::Down => {
                self.position.1 += 1;
            }
        }
    }
}