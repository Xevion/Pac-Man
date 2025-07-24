use std::cell::RefCell;
use std::rc::Rc;

use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::entity::direction::Direction;
use crate::entity::ghost::{Ghost, GhostMode, GhostType};
use crate::entity::pacman::Pacman;
use crate::entity::{Entity, Moving, Renderable, StaticEntity};
use crate::map::Map;
use glam::{IVec2, UVec2};

pub struct Blinky {
    ghost: Ghost,
}

impl Blinky {
    pub fn new(
        starting_position: UVec2,
        body_texture: Texture<'_>,
        eyes_texture: Texture<'_>,
        map: Rc<RefCell<Map>>,
        pacman: Rc<RefCell<Pacman>>,
    ) -> Blinky {
        Blinky {
            ghost: Ghost::new(GhostType::Blinky, starting_position, body_texture, eyes_texture, map, pacman),
        }
    }

    /// Gets Blinky's chase target - directly targets Pac-Man's current position
    pub fn get_chase_target(&self) -> IVec2 {
        let pacman = self.ghost.pacman.borrow();
        let cell = pacman.base().cell_position;
        IVec2::new(cell.x as i32, cell.y as i32)
    }

    pub fn set_mode(&mut self, mode: GhostMode) {
        self.ghost.set_mode(mode);
    }

    pub fn tick(&mut self) {
        self.ghost.tick();
    }
}

impl Entity for Blinky {
    fn base(&self) -> &StaticEntity {
        self.ghost.base.base()
    }
}

impl Renderable for Blinky {
    fn render(&self, canvas: &mut Canvas<Window>) {
        self.ghost.render(canvas);
    }
}

impl Moving for Blinky {
    fn move_forward(&mut self) {
        self.ghost.move_forward();
    }
    fn update_cell_position(&mut self) {
        self.ghost.update_cell_position();
    }
    fn next_cell(&self, direction: Option<Direction>) -> IVec2 {
        self.ghost.next_cell(direction)
    }
    fn is_wall_ahead(&self, direction: Option<Direction>) -> bool {
        self.ghost.is_wall_ahead(direction)
    }
    fn handle_tunnel(&mut self) -> bool {
        self.ghost.handle_tunnel()
    }
    fn is_grid_aligned(&self) -> bool {
        self.ghost.is_grid_aligned()
    }
    fn set_direction_if_valid(&mut self, new_direction: Direction) -> bool {
        self.ghost.set_direction_if_valid(new_direction)
    }
}

// Allow direct access to ghost fields
impl std::ops::Deref for Blinky {
    type Target = Ghost;

    fn deref(&self) -> &Self::Target {
        &self.ghost
    }
}

impl std::ops::DerefMut for Blinky {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ghost
    }
}
