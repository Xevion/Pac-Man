use std::cell::RefCell;
use std::rc::Rc;

use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::entity::MovableEntity;
use crate::{
    entity::Entity,
    ghost::{Ghost, GhostMode, GhostType},
    map::Map,
    pacman::Pacman,
};

pub struct Blinky<'a> {
    ghost: Ghost<'a>,
}

impl<'a> Blinky<'a> {
    pub fn new(
        starting_position: (u32, u32),
        body_texture: Texture<'a>,
        eyes_texture: Texture<'a>,
        map: Rc<RefCell<Map>>,
        pacman: Rc<RefCell<Pacman<'a>>>,
    ) -> Blinky<'a> {
        Blinky {
            ghost: Ghost::new(
                GhostType::Blinky,
                starting_position,
                body_texture,
                eyes_texture,
                map,
                pacman,
            ),
        }
    }

    /// Gets Blinky's chase target - directly targets Pac-Man's current position
    fn get_chase_target(&self) -> (i32, i32) {
        let pacman = self.ghost.pacman.borrow();
        let cell = pacman.base.cell_position;
        (cell.0 as i32, cell.1 as i32)
    }

    pub fn set_mode(&mut self, mode: GhostMode) {
        self.ghost.set_mode(mode);
    }

    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        self.ghost.render(canvas);
    }
}

impl<'a> Entity for Blinky<'a> {
    fn base(&self) -> &MovableEntity {
        self.ghost.base()
    }

    fn is_colliding(&self, other: &dyn Entity) -> bool {
        self.ghost.is_colliding(other)
    }

    fn tick(&mut self) {
        self.ghost.tick()
    }
}

// Allow direct access to ghost fields
impl<'a> std::ops::Deref for Blinky<'a> {
    type Target = Ghost<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ghost
    }
}

impl<'a> std::ops::DerefMut for Blinky<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ghost
    }
}
