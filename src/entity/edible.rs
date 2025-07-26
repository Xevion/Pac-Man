//! Edible entity for Pac-Man: pellets, power pellets, and fruits.
use crate::constants::{FruitType, MapTile, BOARD_CELL_SIZE};
use crate::entity::{Entity, Renderable, StaticEntity};
use crate::map::Map;
use crate::texture::animated::AnimatedTexture;
use crate::texture::blinking::BlinkingTexture;
use anyhow::Result;
use glam::{IVec2, UVec2};
use sdl2::render::WindowCanvas;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdibleKind {
    Pellet,
    PowerPellet,
    Fruit(FruitType),
}

pub enum EdibleSprite {
    Pellet(AnimatedTexture),
    PowerPellet(BlinkingTexture),
}

pub struct Edible {
    pub base: StaticEntity,
    pub kind: EdibleKind,
    pub sprite: EdibleSprite,
}

impl Edible {
    pub fn new_pellet(cell_position: UVec2, sprite: AnimatedTexture) -> Self {
        let pixel_position = Map::cell_to_pixel(cell_position);
        Edible {
            base: StaticEntity::new(pixel_position, cell_position),
            kind: EdibleKind::Pellet,
            sprite: EdibleSprite::Pellet(sprite),
        }
    }
    pub fn new_power_pellet(cell_position: UVec2, sprite: BlinkingTexture) -> Self {
        let pixel_position = Map::cell_to_pixel(cell_position);
        Edible {
            base: StaticEntity::new(pixel_position, cell_position),
            kind: EdibleKind::PowerPellet,
            sprite: EdibleSprite::PowerPellet(sprite),
        }
    }

    /// Checks collision with Pac-Man (or any entity)
    pub fn collide(&self, pacman: &dyn Entity) -> bool {
        self.base.cell_position == pacman.base().cell_position
    }
}

impl Entity for Edible {
    fn base(&self) -> &StaticEntity {
        &self.base
    }
}

impl Renderable for Edible {
    fn render(&mut self, canvas: &mut WindowCanvas) -> Result<()> {
        let pos = self.base.pixel_position;
        let dest = match &mut self.sprite {
            EdibleSprite::Pellet(sprite) => {
                let tile = sprite.current_tile();
                let x = pos.x + ((crate::constants::CELL_SIZE as i32 - tile.size.x as i32) / 2);
                let y = pos.y + ((crate::constants::CELL_SIZE as i32 - tile.size.y as i32) / 2);
                sdl2::rect::Rect::new(x, y, tile.size.x as u32, tile.size.y as u32)
            }
            EdibleSprite::PowerPellet(sprite) => {
                let tile = sprite.animation.current_tile();
                let x = pos.x + ((crate::constants::CELL_SIZE as i32 - tile.size.x as i32) / 2);
                let y = pos.y + ((crate::constants::CELL_SIZE as i32 - tile.size.y as i32) / 2);
                sdl2::rect::Rect::new(x, y, tile.size.x as u32, tile.size.y as u32)
            }
        };

        match &mut self.sprite {
            EdibleSprite::Pellet(sprite) => sprite.render(canvas, dest),
            EdibleSprite::PowerPellet(sprite) => sprite.render(canvas, dest),
        }
    }
}

/// Reconstruct all edibles from the original map layout
pub fn reconstruct_edibles(
    map: Rc<RefCell<Map>>,
    pellet_sprite: AnimatedTexture,
    power_pellet_sprite: BlinkingTexture,
    _fruit_sprite: AnimatedTexture,
) -> Vec<Edible> {
    let mut edibles = Vec::new();
    for x in 0..BOARD_CELL_SIZE.x {
        for y in 0..BOARD_CELL_SIZE.y {
            let tile = map.borrow().get_tile(IVec2::new(x as i32, y as i32));
            match tile {
                Some(MapTile::Pellet) => {
                    edibles.push(Edible::new_pellet(UVec2::new(x, y), pellet_sprite.clone()));
                }
                Some(MapTile::PowerPellet) => {
                    edibles.push(Edible::new_power_pellet(UVec2::new(x, y), power_pellet_sprite.clone()));
                }
                // Fruits can be added here if you have fruit positions
                _ => {}
            }
        }
    }
    edibles
}
