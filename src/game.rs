use sdl2::{pixels::Color, render::Canvas, video::Window};

use crate::constants::{MapTile, BOARD, BOARD_HEIGHT, BOARD_WIDTH};
use crate::pacman::Pacman;
use crate::textures::TextureManager;

pub struct Game<'a> {
    pub textures: TextureManager<'a>,
    canvas: &'a mut Canvas<Window>,
    pacman: Pacman<'a>,
    debug: bool,
}

impl Game<'_> {
    pub fn new<'a>(
        canvas: &'a mut Canvas<Window>,
        texture_manager: TextureManager<'a>,
    ) -> Game<'a> {
        let pacman = Pacman::new(None, &texture_manager.pacman);

        Game {
            canvas,
            textures: texture_manager,
            pacman: pacman,
            debug: true,
        }
    }

    pub fn tick(&mut self) {}

    pub fn draw(&mut self) {
        // Clear the screen (black)
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        
        self.canvas
            .copy(&self.textures.map, None, None)
            .expect("Could not render texture on canvas");

        // Draw a grid
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let tile = BOARD[x as usize][y as usize];
                let color = match tile {
                    MapTile::Empty => None,
                    MapTile::Wall => Some(Color::BLUE),
                    MapTile::Pellet => Some(Color::RED),
                    MapTile::PowerPellet => Some(Color::MAGENTA),
                    MapTile::StartingPosition(_) => Some(Color::GREEN),
                };

                if let Some(color) = color {
                    self.canvas.set_draw_color(color);
                    self.canvas
                        .draw_rect(sdl2::rect::Rect::new(x as i32 * 24, y as i32 * 24, 24, 24))
                        .expect("Could not draw rectangle");
                }
            }
        }

        self.canvas.present();
    }
}
