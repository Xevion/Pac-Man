//! Edible entity for Pac-Man: pellets, power pellets, and fruits.
use crate::constants::{FruitType, MapTile, BOARD_HEIGHT, BOARD_WIDTH};
use crate::direction::Direction;
use crate::entity::{Entity, Renderable, StaticEntity};
use crate::map::Map;
use crate::texture::atlas::AtlasTexture;
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

pub struct Edible<'a> {
    pub base: StaticEntity,
    pub kind: EdibleKind,
    pub sprite: Rc<AtlasTexture<'a>>,
}

impl<'a> Edible<'a> {
    pub fn new(kind: EdibleKind, cell_position: UVec2, sprite: Rc<AtlasTexture<'a>>) -> Self {
        let pixel_position = Map::cell_to_pixel(cell_position);
        Edible {
            base: StaticEntity::new(pixel_position, cell_position),
            kind,
            sprite,
        }
    }

    /// Checks collision with Pac-Man (or any entity)
    pub fn collide(&self, pacman: &dyn Entity) -> bool {
        self.base.is_colliding(pacman)
    }
}

impl<'a> Entity for Edible<'a> {
    fn base(&self) -> &StaticEntity {
        &self.base
    }
}

impl<'a> Renderable for Edible<'a> {
    fn render(&self, canvas: &mut Canvas<Window>) {
        let pos = self.base.pixel_position;
        self.sprite.render(canvas, (pos.x, pos.y), Direction::Right, Some(0));
    }
}

/// Reconstruct all edibles from the original map layout
pub fn reconstruct_edibles<'a>(
    map: Rc<RefCell<Map>>,
    pellet_sprite: Rc<AtlasTexture<'a>>,
    power_pellet_sprite: Rc<AtlasTexture<'a>>,
    _fruit_sprite: Rc<AtlasTexture<'a>>,
) -> Vec<Edible<'a>> {
    let mut edibles = Vec::new();
    for x in 0..BOARD_WIDTH {
        for y in 0..BOARD_HEIGHT {
            let tile = map.borrow().get_tile(IVec2::new(x as i32, y as i32));
            match tile {
                Some(MapTile::Pellet) => {
                    edibles.push(Edible::new(EdibleKind::Pellet, UVec2::new(x, y), Rc::clone(&pellet_sprite)));
                }
                Some(MapTile::PowerPellet) => {
                    edibles.push(Edible::new(
                        EdibleKind::PowerPellet,
                        UVec2::new(x, y),
                        Rc::clone(&power_pellet_sprite),
                    ));
                }
                // Fruits can be added here if you have fruit positions
                _ => {}
            }
        }
    }
    edibles
}
