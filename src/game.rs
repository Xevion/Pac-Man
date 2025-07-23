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

use crate::animation::AtlasTexture;
use crate::audio::Audio;
use crate::constants::RAW_BOARD;
use crate::debug::{DebugMode, DebugRenderer};
use crate::direction::Direction;
use crate::edible::{reconstruct_edibles, Edible, EdibleKind};
use crate::entity::Renderable;
use crate::ghosts::blinky::Blinky;
use crate::map::Map;
use crate::pacman::Pacman;

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
pub struct Game<'a> {
    canvas: &'a mut Canvas<Window>,
    map_texture: Texture<'a>,
    pellet_texture: Rc<AtlasTexture<'a>>,
    power_pellet_texture: Rc<AtlasTexture<'a>>,
    font: Font<'a, 'static>,
    pacman: Rc<RefCell<Pacman<'a>>>,
    map: Rc<RefCell<Map>>,
    debug_mode: DebugMode,
    score: u32,
    audio: Audio,
    blinky: Blinky<'a>,
    edibles: Vec<Edible<'a>>,
}

impl<'a> Game<'a> {
    /// Creates a new `Game` instance.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL canvas to render to.
    /// * `texture_creator` - The SDL texture creator.
    /// * `ttf_context` - The SDL TTF context.
    /// * `_audio_subsystem` - The SDL audio subsystem (currently unused).
    pub fn new(
        canvas: &'a mut Canvas<Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf_context: &'a sdl2::ttf::Sdl2TtfContext,
        _audio_subsystem: &'a sdl2::AudioSubsystem,
    ) -> Game<'a> {
        let map = Rc::new(RefCell::new(Map::new(RAW_BOARD)));

        // Load Pacman texture from embedded data
        let pacman_atlas = texture_creator
            .load_texture_bytes(PACMAN_TEXTURE_DATA)
            .expect("Could not load pacman texture from embedded data");
        let pacman = Rc::new(RefCell::new(Pacman::new(
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
        let pellet_texture = Rc::new(AtlasTexture::new(
            texture_creator
                .load_texture_bytes(PELLET_TEXTURE_DATA)
                .expect("Could not load pellet texture from embedded data"),
            1,
            24,
            24,
            None,
        ));
        let power_pellet_texture = Rc::new(AtlasTexture::new(
            texture_creator
                .load_texture_bytes(POWER_PELLET_TEXTURE_DATA)
                .expect("Could not load power pellet texture from embedded data"),
            1,
            24,
            24,
            None,
        ));

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

        let edibles = reconstruct_edibles(
            Rc::clone(&map),
            Rc::clone(&pellet_texture),
            Rc::clone(&power_pellet_texture),
            Rc::clone(&pellet_texture), // placeholder for fruit sprite
        );

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
            edibles,
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
        if direction.is_some() {
            self.pacman.borrow_mut().next_direction = direction;
            return;
        }

        // Toggle debug mode
        if keycode == Keycode::Space {
            self.debug_mode = match self.debug_mode {
                DebugMode::None => DebugMode::Grid,
                DebugMode::Grid => DebugMode::Pathfinding,
                DebugMode::Pathfinding => DebugMode::ValidPositions,
                DebugMode::ValidPositions => DebugMode::None,
            };
            return;
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

        // Get valid positions from the cached flood fill and randomize positions in a single block
        {
            let mut map = self.map.borrow_mut();
            let valid_positions = map.get_valid_playable_positions();
            let mut rng = rand::rng();

            // Randomize Pac-Man position
            if let Some(pos) = valid_positions.iter().choose(&mut rng) {
                let mut pacman = self.pacman.borrow_mut();
                pacman.base.base.pixel_position = Map::cell_to_pixel((pos.x, pos.y));
                pacman.base.base.cell_position = (pos.x, pos.y);
                pacman.base.in_tunnel = false;
                pacman.base.direction = Direction::Right;
                pacman.next_direction = None;
                pacman.stopped = false;
            }

            // Randomize ghost position
            if let Some(pos) = valid_positions.iter().choose(&mut rng) {
                self.blinky.base.base.pixel_position = Map::cell_to_pixel((pos.x, pos.y));
                self.blinky.base.base.cell_position = (pos.x, pos.y);
                self.blinky.base.in_tunnel = false;
                self.blinky.base.direction = Direction::Left;
                self.blinky.mode = crate::ghost::GhostMode::Chase;
            }
        }

        self.edibles = reconstruct_edibles(
            Rc::clone(&self.map),
            Rc::clone(&self.pellet_texture),
            Rc::clone(&self.power_pellet_texture),
            Rc::clone(&self.pellet_texture), // placeholder for fruit sprite
        );
    }

    /// Advances the game by one tick.
    pub fn tick(&mut self) {
        // Advance animation frames for Pacman and Blinky
        self.pacman.borrow_mut().sprite.tick();
        self.blinky.body_sprite.tick();
        self.blinky.eyes_sprite.tick();

        let pacman = self.pacman.borrow();
        let mut eaten_indices = vec![];
        for (i, edible) in self.edibles.iter().enumerate() {
            if edible.collide(&*pacman) {
                eaten_indices.push(i);
            }
        }
        drop(pacman); // Release immutable borrow before mutably borrowing self
        for &i in eaten_indices.iter().rev() {
            let edible = &self.edibles[i];
            match edible.kind {
                EdibleKind::Pellet => {
                    self.add_score(10);
                    self.audio.eat();
                }
                EdibleKind::PowerPellet => {
                    self.add_score(50);
                    self.audio.eat();
                }
                EdibleKind::Fruit(_fruit) => {
                    self.add_score(100);
                    self.audio.eat();
                }
            }
            self.edibles.remove(i);
        }
        self.pacman.borrow_mut().tick();
        self.blinky.tick();
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

        // Render all edibles
        for edible in &self.edibles {
            edible.render(self.canvas);
        }

        // Render Pac-Man
        self.pacman.borrow().render(self.canvas);

        // Render ghost
        self.blinky.render(self.canvas);

        // Render score
        self.render_ui();

        // Draw the debug grid
        match self.debug_mode {
            DebugMode::Grid => {
                DebugRenderer::draw_debug_grid(
                    self.canvas,
                    &self.map.borrow(),
                    self.pacman.borrow().base.base.cell_position,
                );
                let next_cell =
                    <Pacman as crate::entity::Moving>::next_cell(&*self.pacman.borrow(), None);
                DebugRenderer::draw_next_cell(
                    self.canvas,
                    &self.map.borrow(),
                    (next_cell.0 as u32, next_cell.1 as u32),
                );
            }
            DebugMode::ValidPositions => {
                DebugRenderer::draw_valid_positions(self.canvas, &mut self.map.borrow_mut());
            }
            DebugMode::Pathfinding => {
                DebugRenderer::draw_pathfinding(self.canvas, &self.blinky, &self.map.borrow());
            }
            DebugMode::None => {}
        }

        // Present the canvas
        self.canvas.present();
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
            &format!("{lives}UP   HIGH SCORE   "),
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
