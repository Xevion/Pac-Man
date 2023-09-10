use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::{pixels::Color, render::Canvas, video::Window};

use crate::constants::{MapTile, BOARD, BOARD_HEIGHT, BOARD_WIDTH};
use crate::direction::Direction;
use crate::entity::Entity;
use crate::pacman::Pacman;

pub struct Game<'a> {
    canvas: &'a mut Canvas<Window>,
    map_texture: Texture<'a>,
    pacman: Pacman<'a>,
    debug: bool,
}

impl Game<'_> {
    pub fn new<'a>(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Game<'a> {
        let pacman_atlas = texture_creator
            .load_texture("assets/32/pacman.png")
            .expect("Could not load pacman texture");
        let pacman = Pacman::new(Some(Game::cell_to_pixel((1, 4))), pacman_atlas);

        Game {
            canvas,
            pacman: pacman,
            debug: false,
            map_texture: texture_creator
                .load_texture("assets/map.png")
                .expect("Could not load pacman texture"),
        }
    }

    pub fn cell_to_pixel(cell: (u32, u32)) -> (i32, i32) {
        ((cell.0 as i32 * 24), ((cell.1) as i32 * 24))
    }

    pub fn keyboard_event(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::D => {
                self.pacman.next_direction = Some(Direction::Right);
            }
            Keycode::A => {
                self.pacman.next_direction = Some(Direction::Left);
            }
            Keycode::W => {
                self.pacman.next_direction = Some(Direction::Up);
            }
            Keycode::S => {
                self.pacman.next_direction = Some(Direction::Down);
            }
            Keycode::Space => {
                self.debug = !self.debug;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.pacman.tick();
    }

    pub fn draw(&mut self) {
        // Clear the screen (black)
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas
            .copy(&self.map_texture, None, None)
            .expect("Could not render texture on canvas");

        // Render the pacman
        self.pacman.render(self.canvas);

        // Draw a grid
        if self.debug {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    let tile = BOARD[x as usize][y as usize];
                    let mut color = None;

                    if (x, y) == self.pacman.cell_position() {
                        self.draw_cell((x, y), Color::CYAN);
                    } else {
                        color = match tile {
                            MapTile::Empty => None,
                            MapTile::Wall => Some(Color::BLUE),
                            MapTile::Pellet => Some(Color::RED),
                            MapTile::PowerPellet => Some(Color::MAGENTA),
                            MapTile::StartingPosition(_) => Some(Color::GREEN),
                        };
                    }

                    if let Some(color) = color {
                        self.draw_cell((x, y), color);
                    }
                }
            }
        }

        self.canvas.present();
    }

    fn draw_cell(&mut self, cell: (u32, u32), color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas
            .draw_rect(sdl2::rect::Rect::new(
                cell.0 as i32 * 24,
                cell.1 as i32 * 24,
                24,
                24,
            ))
            .expect("Could not draw rectangle");
    }
}
