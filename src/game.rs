use std::rc::Rc;

use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::{Font, FontStyle};
use sdl2::video::WindowContext;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use tracing::event;

use crate::constants::{MapTile, BOARD_HEIGHT, BOARD_WIDTH, RAW_BOARD};
use crate::direction::Direction;
use crate::entity::Entity;
use crate::map::Map;
use crate::pacman::Pacman;

pub struct Game<'a> {
    canvas: &'a mut Canvas<Window>,
    map_texture: Texture<'a>,
    pellet_texture: Texture<'a>,
    power_pellet_texture: Texture<'a>,
    font: Font<'a, 'static>,
    pacman: Pacman<'a>,
    map: Rc<std::cell::RefCell<Map>>,
    debug: bool,
    score: u32,
}

impl Game<'_> {
    pub fn new<'a>(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
    ) -> Game<'a> {
        let map = Rc::new(std::cell::RefCell::new(Map::new(RAW_BOARD)));
        let pacman_atlas = texture_creator
            .load_texture("assets/32/pacman.png")
            .expect("Could not load pacman texture");
        let pacman = Pacman::new((1, 1), pacman_atlas, Rc::clone(&map));

        let pellet_texture = texture_creator
            .load_texture("assets/24/pellet.png")
            .expect("Could not load pellet texture");
        let power_pellet_texture = texture_creator
            .load_texture("assets/24/energizer.png")
            .expect("Could not load power pellet texture");

        let font = ttf_context
            .load_font("assets/font/konami.ttf", 24)
            .expect("Could not load font");

        Game {
            canvas,
            pacman: pacman,
            debug: false,
            map: map,
            map_texture: texture_creator
                .load_texture("assets/map.png")
                .expect("Could not load map texture"),
            pellet_texture,
            power_pellet_texture,
            font,
            score: 0,
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

    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }

    pub fn tick(&mut self) {
        self.pacman.tick();
        self.check_pellet_eating();
    }

    fn check_pellet_eating(&mut self) {
        let cell_pos = self.pacman.cell_position();

        // Check if there's a pellet at the current position
        let tile = {
            let map = self.map.borrow();
            map.get_tile((cell_pos.0 as i32, cell_pos.1 as i32))
        };

        if let Some(tile) = tile {
            match tile {
                MapTile::Pellet => {
                    // Eat the pellet and add score
                    {
                        let mut map = self.map.borrow_mut();
                        map.set_tile((cell_pos.0 as i32, cell_pos.1 as i32), MapTile::Empty);
                    }
                    self.add_score(10);
                    event!(
                        tracing::Level::DEBUG,
                        "Pellet eaten at ({}, {})",
                        cell_pos.0,
                        cell_pos.1
                    );
                }
                MapTile::PowerPellet => {
                    // Eat the power pellet and add score
                    {
                        let mut map = self.map.borrow_mut();
                        map.set_tile((cell_pos.0 as i32, cell_pos.1 as i32), MapTile::Empty);
                    }
                    self.add_score(50);
                    event!(
                        tracing::Level::DEBUG,
                        "Power pellet eaten at ({}, {})",
                        cell_pos.0,
                        cell_pos.1
                    );
                }
                _ => {}
            }
        }
    }

    pub fn draw(&mut self) {
        // Clear the screen (black)
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Render the map
        self.canvas
            .copy(&self.map_texture, None, None)
            .expect("Could not render texture on canvas");

        // Render pellets
        self.render_pellets();

        // Render the pacman
        self.pacman.render(self.canvas);

        // Render score
        self.render_score();

        // Draw the debug grid
        if self.debug {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    let tile = self
                        .map
                        .borrow()
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

    fn render_pellets(&mut self) {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let tile = self
                    .map
                    .borrow()
                    .get_tile((x as i32, y as i32))
                    .unwrap_or(MapTile::Empty);

                match tile {
                    MapTile::Pellet => {
                        let position = Map::cell_to_pixel((x, y));
                        let dst_rect = sdl2::rect::Rect::new(position.0, position.1, 24, 24);
                        self.canvas
                            .copy(&self.pellet_texture, None, Some(dst_rect))
                            .expect("Could not render pellet");
                    }
                    MapTile::PowerPellet => {
                        let position = Map::cell_to_pixel((x, y));
                        let dst_rect = sdl2::rect::Rect::new(position.0, position.1, 24, 24);
                        self.canvas
                            .copy(&self.power_pellet_texture, None, Some(dst_rect))
                            .expect("Could not render power pellet");
                    }
                    _ => {}
                }
            }
        }
    }

    fn render_score(&mut self) {
        let lives = 3;
        let score_text = format!("{:02}", self.score);

        let x_offset = 12;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        let gap_offset = 6;

        self.render_text(
            &format!("{}UP   HIGH SCORE   ", lives),
            (24 * lives_offset + x_offset, y_offset),
            Color::WHITE,
        );
        self.render_text(
            &score_text,
            (24 * score_offset + x_offset, 24 + y_offset + gap_offset),
            Color::WHITE,
        );
    }

    fn render_text(&mut self, text: &str, position: (i32, i32), color: Color) {
        let surface = self
            .font
            .render(text)
            .blended(color)
            .expect("Could not render text surface");

        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .expect("Could not create texture from surface");

        let query = texture.query();

        let dst_rect =
            sdl2::rect::Rect::new(position.0, position.1, query.width + 4, query.height + 4);

        self.canvas
            .copy(&texture, None, Some(dst_rect))
            .expect("Could not render text texture");
    }
}
