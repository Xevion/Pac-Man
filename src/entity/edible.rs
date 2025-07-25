//! Edible entity for Pac-Man: pellets, power pellets, and fruits.
use crate::constants::{FruitType, MapTile, BOARD_HEIGHT, BOARD_WIDTH};
use crate::entity::direction::Direction;
use crate::entity::{Entity, Renderable, StaticEntity};
use crate::map::Map;
use crate::texture::atlas::AtlasTexture;
use crate::texture::blinking::BlinkingTexture;
use crate::texture::FrameDrawn;
use glam::{IVec2, UVec2};
use sdl2::{render::Canvas, video::Window};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdibleKind {
    Pellet,
    PowerPellet,
    Fruit(FruitType),
}

pub enum EdibleSprite {
    Pellet(Rc<Box<dyn FrameDrawn>>),
    PowerPellet(Rc<RefCell<BlinkingTexture>>),
}

pub struct Edible {
    pub base: StaticEntity,
    pub kind: EdibleKind,
    pub sprite: EdibleSprite,
}

impl Edible {
    pub fn new_pellet(cell_position: UVec2, sprite: Rc<Box<dyn FrameDrawn>>) -> Self {
        let pixel_position = Map::cell_to_pixel(cell_position);
        Edible {
            base: StaticEntity::new(pixel_position, cell_position),
            kind: EdibleKind::Pellet,
            sprite: EdibleSprite::Pellet(sprite),
        }
    }
    pub fn new_power_pellet(cell_position: UVec2, sprite: Rc<RefCell<BlinkingTexture>>) -> Self {
        let pixel_position = Map::cell_to_pixel(cell_position);
        Edible {
            base: StaticEntity::new(pixel_position, cell_position),
            kind: EdibleKind::PowerPellet,
            sprite: EdibleSprite::PowerPellet(sprite),
        }
    }

    /// Checks collision with Pac-Man (or any entity)
    pub fn collide(&self, pacman: &dyn Entity) -> bool {
        self.base.is_colliding(pacman)
    }
}

impl Entity for Edible {
    fn base(&self) -> &StaticEntity {
        &self.base
    }
}

impl Renderable for Edible {
    fn render(&self, canvas: &mut Canvas<Window>) {
        let pos = self.base.pixel_position;
        match &self.sprite {
            EdibleSprite::Pellet(sprite) => sprite.render(canvas, pos, Direction::Right, Some(0)),
            EdibleSprite::PowerPellet(sprite) => sprite.borrow().render(canvas, pos, Direction::Right, Some(0)),
        }
    }
}

/// Reconstruct all edibles from the original map layout
pub fn reconstruct_edibles(
    map: Rc<RefCell<Map>>,
    pellet_sprite: Rc<Box<dyn FrameDrawn>>,
    power_pellet_sprite: Rc<RefCell<BlinkingTexture>>,
    _fruit_sprite: Rc<Box<dyn FrameDrawn>>,
) -> Vec<Edible> {
    let mut edibles = Vec::new();
    for x in 0..BOARD_WIDTH {
        for y in 0..BOARD_HEIGHT {
            let tile = map.borrow().get_tile(IVec2::new(x as i32, y as i32));
            match tile {
                Some(MapTile::Pellet) => {
                    edibles.push(Edible::new_pellet(UVec2::new(x, y), Rc::clone(&pellet_sprite)));
                }
                Some(MapTile::PowerPellet) => {
                    edibles.push(Edible::new_power_pellet(UVec2::new(x, y), Rc::clone(&power_pellet_sprite)));
                }
                // Fruits can be added here if you have fruit positions
                _ => {}
            }
        }
    }
    edibles
}
