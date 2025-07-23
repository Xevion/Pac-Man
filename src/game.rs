//! This module contains the main game logic and state.
use std::cell::RefCell;
use std::rc::Rc;

use rand::seq::IteratorRandom;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::{pixels::Color, render::Canvas, video::Window};
use tracing::event;

use crate::audio::Audio;
use crate::{
    constants::{MapTile, BOARD_HEIGHT, BOARD_WIDTH, RAW_BOARD},
    direction::Direction,
    entity::{Entity, Renderable},
    ghosts::blinky::Blinky,
    map::Map,
    pacman::Pacman,
};

// Embed texture data directly into the executable
static PACMAN_TEXTURE_DATA: &[u8] = include_bytes!("../assets/32/pacman.png");
static PELLET_TEXTURE_DATA: &[u8] = include_bytes!("../assets/24/pellet.png");
static POWER_PELLET_TEXTURE_DATA: &[u8] = include_bytes!("../assets/24/energizer.png");
static MAP_TEXTURE_DATA: &[u8] = include_bytes!("../assets/map.png");
static FONT_DATA: &[u8] = include_bytes!("../assets/font/konami.ttf");

// Add ghost texture data
static GHOST_BODY_TEXTURE_DATA: &[u8] = include_bytes!("../assets/32/ghost_body.png");
static GHOST_EYES_TEXTURE_DATA: &[u8] = include_bytes!("../assets/32/ghost_eyes.png");

/// The main game state.
///
/// This struct contains all the information necessary to run the game, including
/// the canvas, textures, fonts, game objects, and the current score.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DebugMode {
    None,
    Grid,
    Pathfinding,
    ValidPositions,
}

pub struct Game<'a> {
    canvas: &'a mut Canvas<Window>,
    map_texture: Texture<'a>,
    pellet_texture: Texture<'a>,
    power_pellet_texture: Texture<'a>,
    font: Font<'a, 'static>,
    pacman: Rc<RefCell<Pacman<'a>>>,
    map: Rc<std::cell::RefCell<Map>>,
    debug_mode: DebugMode,
    score: u32,
    audio: crate::audio::Audio,
    // Add ghost
    blinky: Blinky<'a>,
}

impl Game<'_> {
    /// Creates a new `Game` instance.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL canvas to render to.
    /// * `texture_creator` - The SDL texture creator.
    /// * `ttf_context` - The SDL TTF context.
    /// * `_audio_subsystem` - The SDL audio subsystem (currently unused).
    pub fn new<'a>(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
        _audio_subsystem: &'a sdl2::AudioSubsystem,
    ) -> Game<'a> {
        let map = Rc::new(std::cell::RefCell::new(Map::new(RAW_BOARD)));

        // Load Pacman texture from embedded data
        let pacman_atlas = texture_creator
            .load_texture_bytes(PACMAN_TEXTURE_DATA)
            .expect("Could not load pacman texture from embedded data");
        let pacman = Rc::new(std::cell::RefCell::new(Pacman::new(
            (1, 1),
            pacman_atlas,
            Rc::clone(&map),
        )));

        // Load ghost textures
        let ghost_body = texture_creator
            .load_texture_bytes(GHOST_BODY_TEXTURE_DATA)
            .expect("Could not load ghost body texture from embedded data");
        let ghost_eyes = texture_creator
            .load_texture_bytes(GHOST_EYES_TEXTURE_DATA)
            .expect("Could not load ghost eyes texture from embedded data");

        // Create Blinky
        let blinky = Blinky::new(
            (13, 11), // Starting position just above ghost house
            ghost_body,
            ghost_eyes,
            Rc::clone(&map),
            Rc::clone(&pacman),
        );

        // Load pellet texture from embedded data
        let pellet_texture = texture_creator
            .load_texture_bytes(PELLET_TEXTURE_DATA)
            .expect("Could not load pellet texture from embedded data");

        // Load power pellet texture from embedded data
        let power_pellet_texture = texture_creator
            .load_texture_bytes(POWER_PELLET_TEXTURE_DATA)
            .expect("Could not load power pellet texture from embedded data");

        // Load font from embedded data
        let font_rwops = RWops::from_bytes(FONT_DATA).expect("Failed to create RWops for font");
        let font = ttf_context
            .load_font_from_rwops(font_rwops, 24)
            .expect("Could not load font from embedded data");

        let audio = Audio::new();

        // Load map texture from embedded data
        let mut map_texture = texture_creator
            .load_texture_bytes(MAP_TEXTURE_DATA)
            .expect("Could not load map texture from embedded data");
        map_texture.set_color_mod(0, 0, 255);

        Game {
            canvas,
            pacman,
            debug_mode: DebugMode::None,
            map,
            map_texture,
            pellet_texture,
            power_pellet_texture,
            font,
            score: 0,
            audio,
            blinky,
        }
    }

    /// Handles a keyboard event.
    ///
    /// # Arguments
    ///
    /// * `keycode` - The keycode of the key that was pressed.
    pub fn keyboard_event(&mut self, keycode: Keycode) {
        // Change direction
        let direction = Direction::from_keycode(keycode);
        self.pacman.borrow_mut().next_direction = direction;

        // Toggle debug mode
        if keycode == Keycode::Space {
            self.debug_mode = match self.debug_mode {
                DebugMode::None => DebugMode::Grid,
                DebugMode::Grid => DebugMode::Pathfinding,
                DebugMode::Pathfinding => DebugMode::ValidPositions,
                DebugMode::ValidPositions => DebugMode::None,
            };
        }

        // Reset game
        if keycode == Keycode::R {
            self.reset();
        }
    }

    /// Adds points to the score.
    ///
    /// # Arguments
    ///
    /// * `points` - The number of points to add.
    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }

    /// Resets the game to its initial state.
    pub fn reset(&mut self) {
        // Reset the map to restore all pellets
        {
            let mut map = self.map.borrow_mut();
            map.reset();
        }

        // Reset the score
        self.score = 0;

        // Get valid positions from the cached flood fill
        let mut map = self.map.borrow_mut();
        let valid_positions = map.get_valid_playable_positions();
        let mut rng = rand::rng();

        // Randomize Pac-Man position
        if let Some(pos) = valid_positions.iter().choose(&mut rng) {
            let mut pacman = self.pacman.borrow_mut();
            pacman.base.pixel_position = Map::cell_to_pixel((pos.x, pos.y));
            pacman.base.cell_position = (pos.x, pos.y);
            pacman.base.in_tunnel = false;
            pacman.base.direction = Direction::Right;
            pacman.next_direction = None;
            pacman.stopped = false;
        }

        // Randomize ghost position
        if let Some(pos) = valid_positions.iter().choose(&mut rng) {
            self.blinky.base.pixel_position = Map::cell_to_pixel((pos.x, pos.y));
            self.blinky.base.cell_position = (pos.x, pos.y);
            self.blinky.base.in_tunnel = false;
            self.blinky.base.direction = Direction::Left;
            self.blinky.mode = crate::ghost::GhostMode::Chase;
        }
    }

    /// Advances the game by one tick.
    pub fn tick(&mut self) {
        self.check_pellet_eating();
        self.pacman.borrow_mut().tick();
        self.blinky.tick();
    }

    /// Checks if Pac-Man is currently eating a pellet and updates the game state
    /// accordingly.
    fn check_pellet_eating(&mut self) {
        let cell_pos = self.pacman.borrow().base.cell_position;

        // Check if there's a pellet at the current position
        let tile = {
            let map = self.map.borrow();
            map.get_tile((cell_pos.0 as i32, cell_pos.1 as i32))
        };

        if let Some(tile) = tile {
            let pellet_value = match tile {
                MapTile::Pellet => Some(10),
                MapTile::PowerPellet => Some(50),
                _ => None,
            };

            if let Some(value) = pellet_value {
                {
                    let mut map = self.map.borrow_mut();
                    map.set_tile((cell_pos.0 as i32, cell_pos.1 as i32), MapTile::Empty);
                }
                self.add_score(value);
                self.audio.eat();
                event!(
                    tracing::Level::DEBUG,
                    "Pellet eaten at ({}, {})",
                    cell_pos.0,
                    cell_pos.1
                );
            }
        }
    }

    /// Draws the entire game to the canvas.
    pub fn draw(&mut self) {
        // Clear the screen (black)
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Render the map
        self.canvas
            .copy(&self.map_texture, None, None)
            .expect("Could not render texture on canvas");

        // Render pellets
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let tile = self
                    .map
                    .borrow()
                    .get_tile((x as i32, y as i32))
                    .unwrap_or(MapTile::Empty);

                let texture = match tile {
                    MapTile::Pellet => Some(&self.pellet_texture),
                    MapTile::PowerPellet => Some(&self.power_pellet_texture),
                    _ => None,
                };

                if let Some(texture) = texture {
                    let position = Map::cell_to_pixel((x, y));
                    let dst_rect = sdl2::rect::Rect::new(position.0, position.1, 24, 24);
                    self.canvas
                        .copy(texture, None, Some(dst_rect))
                        .expect("Could not render pellet");
                }
            }
        }

        // Render Pac-Man
        self.pacman.borrow_mut().render(self.canvas);

        // Render ghost
        self.blinky.render(self.canvas);

        // Render score
        self.render_ui();

        // Draw the debug grid
        if self.debug_mode == DebugMode::Grid {
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    let tile = self
                        .map
                        .borrow()
                        .get_tile((x as i32, y as i32))
                        .unwrap_or(MapTile::Empty);
                    let mut color = None;

                    if (x, y) == self.pacman.borrow().base.cell_position {
                        self.draw_cell((x, y), Color::CYAN);
                    } else {
                        color = match tile {
                            MapTile::Empty => None,
                            MapTile::Wall => Some(Color::BLUE),
                            MapTile::Pellet => Some(Color::RED),
                            MapTile::PowerPellet => Some(Color::MAGENTA),
                            MapTile::StartingPosition(_) => Some(Color::GREEN),
                            MapTile::Tunnel => Some(Color::CYAN),
                        };
                    }

                    if let Some(color) = color {
                        self.draw_cell((x, y), color);
                    }
                }
            }

            // Draw the next cell
            let next_cell = self.pacman.borrow().base.next_cell(None);
            self.draw_cell((next_cell.0 as u32, next_cell.1 as u32), Color::YELLOW);
        }

        // Show valid playable positions
        if self.debug_mode == DebugMode::ValidPositions {
            let valid_positions_vec = {
                let mut map = self.map.borrow_mut();
                map.get_valid_playable_positions().clone()
            };
            for &pos in &valid_positions_vec {
                self.draw_cell((pos.x, pos.y), Color::RGB(255, 140, 0)); // ORANGE
            }
        }

        // Pathfinding debug mode
        if self.debug_mode == DebugMode::Pathfinding {
            // Show the current path for Blinky
            if let Some((path, _)) = self.blinky.get_path_to_target({
                let (tx, ty) = self.blinky.get_target_tile();
                (tx as u32, ty as u32)
            }) {
                for &(x, y) in &path {
                    self.draw_cell((x, y), Color::YELLOW);
                }
            }
        }

        // Present the canvas
        self.canvas.present();
    }

    /// Draws a single cell to the canvas with the given color.
    ///
    /// # Arguments
    ///
    /// * `cell` - The cell to draw, in grid coordinates.
    /// * `color` - The color to draw the cell with.
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

    /// Renders the user interface, including the score and lives.
    fn render_ui(&mut self) {
        let lives = 3;
        let score_text = format!("{:02}", self.score);

        let x_offset = 12;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        let gap_offset = 6;

        // Render the score and high score
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

    /// Renders text to the screen at the given position.
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

        let dst_rect = sdl2::rect::Rect::new(position.0, position.1, query.width, query.height);

        self.canvas
            .copy(&texture, None, Some(dst_rect))
            .expect("Could not render text texture");
    }
}
