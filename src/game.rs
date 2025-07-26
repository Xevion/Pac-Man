//! This module contains the main game logic and state.
use std::cell::RefCell;
use std::ops::Not;
use std::rc::Rc;

use anyhow::Result;
use glam::{IVec2, UVec2};
use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::{pixels::Color, render::Canvas, video::Window};

use crate::asset::{get_asset_bytes, Asset};
use crate::audio::Audio;
use crate::constants::RAW_BOARD;
use crate::debug::{DebugMode, DebugRenderer};
use crate::entity::blinky::Blinky;
use crate::entity::direction::Direction;
use crate::entity::edible::{reconstruct_edibles, Edible, EdibleKind};
use crate::entity::pacman::Pacman;
use crate::entity::Renderable;
use crate::map::Map;
use crate::texture::animated::AnimatedTexture;
use crate::texture::blinking::BlinkingTexture;
use crate::texture::sprite::{AtlasMapper, AtlasTile, SpriteAtlas};
use crate::texture::{get_atlas_tile, sprite};

/// The main game state.
///
/// Contains all the information necessary to run the game, including
/// the game state, rendering resources, and audio.
pub struct Game {
    // Game state
    pacman: Rc<RefCell<Pacman>>,
    blinky: Blinky,
    edibles: Vec<Edible>,
    map: Rc<RefCell<Map>>,
    score: u32,
    debug_mode: DebugMode,

    // FPS tracking
    fps_1s: f64,
    fps_10s: f64,

    // Rendering resources
    atlas: Rc<SpriteAtlas>,
    font: Font<'static, 'static>,
    map_texture: AtlasTile,

    // Audio
    pub audio: Audio,
}

impl Game {
    /// Creates a new `Game` instance.
    pub fn new(
        texture_creator: &TextureCreator<WindowContext>,
        ttf_context: &sdl2::ttf::Sdl2TtfContext,
        _audio_subsystem: &sdl2::AudioSubsystem,
    ) -> Game {
        let map = Rc::new(RefCell::new(Map::new(RAW_BOARD)));
        let atlas_bytes = get_asset_bytes(Asset::Atlas).expect("Failed to load asset");
        let atlas_texture = unsafe {
            sprite::texture_to_static(
                texture_creator
                    .load_texture_bytes(&atlas_bytes)
                    .expect("Could not load atlas texture from asset API"),
            )
        };
        let atlas_json = get_asset_bytes(Asset::AtlasJson).expect("Failed to load asset");
        let atlas_mapper: AtlasMapper = serde_json::from_slice(&atlas_json).expect("Could not parse atlas JSON");
        let atlas = Rc::new(SpriteAtlas::new(atlas_texture, atlas_mapper));
        let pacman = Rc::new(RefCell::new(Pacman::new(
            UVec2::new(1, 1),
            Rc::clone(&atlas),
            Rc::clone(&map),
        )));
        let blinky = Blinky::new(UVec2::new(13, 11), Rc::clone(&atlas), Rc::clone(&map), Rc::clone(&pacman));
        let map_texture = get_atlas_tile(&atlas, "maze/full.png");
        let edibles = reconstruct_edibles(
            Rc::clone(&map),
            AnimatedTexture::new(vec![get_atlas_tile(&atlas, "maze/pellet.png")], 0),
            BlinkingTexture::new(
                AnimatedTexture::new(vec![get_atlas_tile(&atlas, "maze/energizer.png")], 0),
                17,
                17,
            ),
            AnimatedTexture::new(vec![get_atlas_tile(&atlas, "edible/cherry.png")], 0),
        );
        let font = {
            let font_bytes = get_asset_bytes(Asset::FontKonami).expect("Failed to load asset").into_owned();
            let font_bytes_static: &'static [u8] = Box::leak(font_bytes.into_boxed_slice());
            let font_rwops = RWops::from_bytes(font_bytes_static).expect("Failed to create RWops for font");
            let ttf_context_static: &'static sdl2::ttf::Sdl2TtfContext = unsafe { std::mem::transmute(ttf_context) };
            ttf_context_static
                .load_font_from_rwops(font_rwops, 24)
                .expect("Could not load font from asset API")
        };
        let audio = Audio::new();
        Game {
            pacman,
            blinky,
            edibles,
            map,
            score: 0,
            debug_mode: DebugMode::None,
            atlas,
            font,
            map_texture,
            audio,
            fps_1s: 0.0,
            fps_10s: 0.0,
        }
    }

    /// Handles a keyboard event.
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

        // Toggle mute
        if keycode == Keycode::M {
            self.audio.set_mute(self.audio.is_muted().not());
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

    /// Updates the FPS tracking values.
    pub fn update_fps(&mut self, fps_1s: f64, fps_10s: f64) {
        self.fps_1s = fps_1s;
        self.fps_10s = fps_10s;
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
            let mut rng = SmallRng::from_os_rng();

            // Randomize Pac-Man position
            if let Some(pos) = valid_positions.iter().choose(&mut rng) {
                let mut pacman = self.pacman.borrow_mut();
                pacman.base.base.pixel_position = Map::cell_to_pixel(*pos);
                pacman.base.base.cell_position = *pos;
                pacman.base.in_tunnel = false;
                pacman.base.direction = Direction::Right;
                pacman.next_direction = None;
                pacman.stopped = false;
            }

            // Randomize ghost position
            if let Some(pos) = valid_positions.iter().choose(&mut rng) {
                self.blinky.base.base.pixel_position = Map::cell_to_pixel(*pos);
                self.blinky.base.base.cell_position = *pos;
                self.blinky.base.in_tunnel = false;
                self.blinky.base.direction = Direction::Left;
                self.blinky.mode = crate::entity::ghost::GhostMode::Chase;
            }
        }

        self.edibles = reconstruct_edibles(
            Rc::clone(&self.map),
            AnimatedTexture::new(vec![get_atlas_tile(&self.atlas, "maze/pellet.png")], 0),
            BlinkingTexture::new(
                AnimatedTexture::new(vec![get_atlas_tile(&self.atlas, "maze/energizer.png")], 0),
                12,
                12,
            ),
            AnimatedTexture::new(vec![get_atlas_tile(&self.atlas, "edible/cherry.png")], 0),
        );
    }

    /// Advances the game by one tick.
    pub fn tick(&mut self) {
        self.tick_entities();
        self.handle_edible_collisions();
        self.tick_entities();
    }
    fn tick_entities(&mut self) {
        self.pacman.borrow_mut().tick();
        self.blinky.tick();
        for edible in self.edibles.iter_mut() {
            if let EdibleKind::PowerPellet = edible.kind {
                if let crate::entity::edible::EdibleSprite::PowerPellet(texture) = &mut edible.sprite {
                    texture.tick();
                }
            }
        }
    }
    fn handle_edible_collisions(&mut self) {
        let pacman = self.pacman.borrow();
        let mut eaten_indices = vec![];
        for (i, edible) in self.edibles.iter().enumerate() {
            if edible.collide(&*pacman) {
                eaten_indices.push(i);
            }
        }
        drop(pacman);
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
            // Set Pac-Man to skip the next movement tick
            self.pacman.borrow_mut().skip_move_tick = true;
        }
    }

    /// Draws the entire game to the canvas using a backbuffer.
    pub fn draw(&mut self, window_canvas: &mut Canvas<Window>, backbuffer: &mut Texture) -> Result<()> {
        let texture_creator = window_canvas.texture_creator();
        window_canvas
            .with_texture_canvas(backbuffer, |texture_canvas| {
                let this = self as *mut Self;
                let this = unsafe { &mut *this };
                texture_canvas.set_draw_color(Color::BLACK);
                texture_canvas.clear();
                this.map.borrow_mut().render(texture_canvas, &this.map_texture);
                for edible in this.edibles.iter_mut() {
                    let _ = edible.render(texture_canvas);
                }
                let _ = this.pacman.borrow_mut().render(texture_canvas);
                let _ = this.blinky.render(texture_canvas);
                this.render_ui_on(texture_canvas, &texture_creator);
                match this.debug_mode {
                    DebugMode::Grid => {
                        DebugRenderer::draw_debug_grid(
                            texture_canvas,
                            &this.map.borrow(),
                            this.pacman.borrow().base.base.cell_position,
                        );
                        let next_cell = <Pacman as crate::entity::Moving>::next_cell(&*this.pacman.borrow(), None);
                        DebugRenderer::draw_next_cell(texture_canvas, &this.map.borrow(), next_cell.as_uvec2());
                    }
                    DebugMode::ValidPositions => {
                        DebugRenderer::draw_valid_positions(texture_canvas, &mut this.map.borrow_mut());
                    }
                    DebugMode::Pathfinding => {
                        DebugRenderer::draw_pathfinding(texture_canvas, &this.blinky, &this.map.borrow());
                    }
                    DebugMode::None => {}
                }
            })
            .map_err(|e| anyhow::anyhow!(format!("Failed to render to backbuffer: {e}")))
    }
    pub fn present_backbuffer(&self, canvas: &mut Canvas<Window>, backbuffer: &Texture) -> Result<()> {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.copy(backbuffer, None, None).map_err(anyhow::Error::msg)?;
        canvas.present();
        Ok(())
    }

    fn render_ui_on<C: sdl2::render::RenderTarget>(
        &mut self,
        canvas: &mut sdl2::render::Canvas<C>,
        texture_creator: &TextureCreator<WindowContext>,
    ) {
        let lives = 3;
        let score_text = format!("{:02}", self.score);
        let x_offset = 12;
        let y_offset = 2;
        let lives_offset = 3;
        let score_offset = 7 - (score_text.len() as i32);
        let gap_offset = 6;
        self.render_text_on(
            canvas,
            &*texture_creator,
            &format!("{lives}UP   HIGH SCORE   "),
            IVec2::new(24 * lives_offset + x_offset, y_offset),
            Color::WHITE,
        );
        self.render_text_on(
            canvas,
            &*texture_creator,
            &score_text,
            IVec2::new(24 * score_offset + x_offset, 24 + y_offset + gap_offset),
            Color::WHITE,
        );

        // Display FPS information in top-left corner
        // let fps_text = format!("FPS: {:.1} (1s) / {:.1} (10s)", self.fps_1s, self.fps_10s);
        // self.render_text_on(
        //     canvas,
        //     &*texture_creator,
        //     &fps_text,
        //     IVec2::new(10, 10),
        //     Color::RGB(255, 255, 0), // Yellow color for FPS display
        // );
    }

    fn render_text_on<C: sdl2::render::RenderTarget>(
        &mut self,
        canvas: &mut sdl2::render::Canvas<C>,
        texture_creator: &TextureCreator<WindowContext>,
        text: &str,
        position: IVec2,
        color: Color,
    ) {
        let surface = self.font.render(text).blended(color).expect("Could not render text surface");
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .expect("Could not create texture from surface");
        let query = texture.query();
        let dst_rect = sdl2::rect::Rect::new(position.x, position.y, query.width, query.height);
        canvas
            .copy(&texture, None, Some(dst_rect))
            .expect("Could not render text texture");
    }
}
