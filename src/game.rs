use std::rc::Rc;

use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;
use sdl2::{pixels::Color, render::Canvas, video::Window};

use crate::constants::{MapTile, BOARD_HEIGHT, BOARD_WIDTH, RAW_BOARD};
use crate::direction::Direction;
use crate::entity::Entity;
use crate::map::Map;
use crate::pacman::Pacman;

pub struct Game<'a> {
    canvas: &'a mut Canvas<Window>,
    map_texture: Texture<'a>,
    pacman: Pacman<'a>,
    map: Rc<Map>,
    debug: bool,
}

impl Game<'_> {
    pub fn new<'a>(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Game<'a> {
        let map = Rc::new(Map::new(RAW_BOARD));
        let pacman_atlas = texture_creator
            .load_texture("assets/32/pacman.png")
            .expect("Could not load pacman texture");
        let pacman = Pacman::new((1, 1), pacman_atlas, Rc::clone(&map));

        Game {
            canvas,
            pacman: pacman,
            debug: false,
            map: map,
            map_texture: texture_creator
                .load_texture("assets/map.png")
                .expect("Could not load pacman texture"),
        }
    }

    pub fn keyboard_event(&mut self, keycode: Keycode) {
        // Change direction
        let direction = Direction::from_keycode(keycode);
        self.pacman.next_direction = direction;

        // Toggle debug mode
        if keycode == Keycode::Space {
            self.debug = !self.debug;
        }
    }

    pub fn tick(&mut self) {
        self.pacman.tick();
    }

    pub fn draw(&mut self) {
        // Clear the screen (black)
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();


        // Render the map   
        self.canvas
            .copy(&self.map_texture, None, None)
            .expect("Could not render texture on canvas");

        // Render the pacman
        self.pacman.render(self.canvas);

        // Draw a grid
        if self.debug {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    let tile = self
                        .map
                        .get_tile((x as i32, y as i32))
                        .unwrap_or(MapTile::Empty);
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

            // Draw the next cell
            let next_cell = self.pacman.next_cell(None);
            self.draw_cell((next_cell.0 as u32, next_cell.1 as u32), Color::YELLOW);
        }

        // Present the canvas
        self.canvas.present();
    }

    fn draw_cell(&mut self, cell: (u32, u32), color: Color) {
        let position = Map::cell_to_pixel(cell);
        self.canvas.set_draw_color(color);
        self.canvas
            .draw_rect(sdl2::rect::Rect::new(
                position.0 as i32,
                position.1 as i32,
                24,
                24,
            ))
            .expect("Could not draw rectangle");
    }
}
