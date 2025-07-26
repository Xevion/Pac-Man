//! This module defines the Pac-Man entity, including its behavior and rendering.
use anyhow::Result;
use glam::{IVec2, UVec2};
use sdl2::render::WindowCanvas;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    entity::speed::SimpleTickModulator,
    entity::{direction::Direction, Entity, MovableEntity, Moving, QueuedDirection, Renderable, StaticEntity},
    map::Map,
    texture::{animated::AnimatedTexture, directional::DirectionalAnimatedTexture, get_atlas_tile, sprite::SpriteAtlas},
};

/// The Pac-Man entity.
pub struct Pacman {
    /// Shared movement and position fields.
    pub base: MovableEntity,
    /// The next direction of Pac-Man, which will be applied when Pac-Man is next aligned with the grid.
    pub next_direction: Option<Direction>,
    /// Whether Pac-Man is currently stopped.
    pub stopped: bool,
    pub skip_move_tick: bool,
    pub texture: DirectionalAnimatedTexture,
    pub death_animation: AnimatedTexture,
}

impl Entity for Pacman {
    fn base(&self) -> &StaticEntity {
        &self.base.base
    }
}

impl Moving for Pacman {
    fn tick_movement(&mut self) {
        if self.skip_move_tick {
            self.skip_move_tick = false;
            return;
        }
        self.base.tick_movement();
    }
    fn update_cell_position(&mut self) {
        self.base.update_cell_position();
    }
    fn next_cell(&self, direction: Option<Direction>) -> IVec2 {
        self.base.next_cell(direction)
    }
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        self.base.is_wall_ahead(direction)
    }
    fn handle_tunnel(&mut self) -> bool {
        self.base.handle_tunnel()
    }
    fn is_grid_aligned(&self) -> bool {
        self.base.is_grid_aligned()
    }
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
        self.base.set_direction_if_valid(new_direction)
    }
    fn on_grid_aligned(&mut self) {
        Pacman::update_cell_position(self);
        if !<Pacman as Moving>::handle_tunnel(self) {
            <Pacman as QueuedDirection>::handle_direction_change(self);
            if !self.stopped && <Pacman as Moving>::is_wall_ahead(self, None) {
                self.stopped = true;
            } else if self.stopped && !<Pacman as Moving>::is_wall_ahead(self, None) {
                self.stopped = false;
            }
        }
    }
}

impl QueuedDirection for Pacman {
    fn next_direction(&self) -> Option<Direction> {
        self.next_direction
    }
    fn set_next_direction(&mut self, dir: Option<Direction>) {
        self.next_direction = dir;
    }
}

impl Pacman {
    /// Creates a new `Pacman` instance.
    pub fn new(starting_position: UVec2, atlas: Rc<RefCell<SpriteAtlas>>, map: Rc<RefCell<Map>>) -> Pacman {
        let pixel_position = Map::cell_to_pixel(starting_position);
        let get = |name: &str| get_atlas_tile(&atlas, name);

        Pacman {
            base: MovableEntity::new(
                pixel_position,
                starting_position,
                Direction::Right,
                SimpleTickModulator::new(1f32),
                map,
            ),
            next_direction: None,
            stopped: false,
            skip_move_tick: false,
            texture: DirectionalAnimatedTexture::new(
                vec![get("pacman/up_a.png"), get("pacman/up_b.png"), get("pacman/full.png")],
                vec![get("pacman/down_a.png"), get("pacman/down_b.png"), get("pacman/full.png")],
                vec![get("pacman/left_a.png"), get("pacman/left_b.png"), get("pacman/full.png")],
                vec![get("pacman/right_a.png"), get("pacman/right_b.png"), get("pacman/full.png")],
                8,
            ),
            death_animation: AnimatedTexture::new(
                (0..=10)
                    .map(|i| get_atlas_tile(&atlas, &format!("pacman/death/{}.png", i)))
                    .collect(),
                5,
            ),
        }
    }

    /// Returns the internal position of Pac-Man, rounded down to the nearest even number.
    fn internal_position_even(&self) -> UVec2 {
        let pos = self.base.internal_position();
        UVec2::new((pos.x / 2) * 2, (pos.y / 2) * 2)
    }

    pub fn tick(&mut self) {
        <Pacman as Moving>::tick(self);
        self.texture.tick();
    }
}

impl Renderable for Pacman {
    fn render(&mut self, canvas: &mut WindowCanvas) -> Result<()> {
        let pos = self.base.base.pixel_position;
        let dir = self.base.direction;

        // Center the 16x16 sprite on the 8x8 cell by offsetting by -4
        let dest = sdl2::rect::Rect::new(pos.x - 4, pos.y - 4, 16, 16);

        if self.stopped {
            // When stopped, show the full sprite (mouth open)
            self.texture.render_stopped(canvas, dest, dir)?;
        } else {
            self.texture.render(canvas, dest, dir)?;
        }
        return Ok(());
    }
}
