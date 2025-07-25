use glam::IVec2;
use sdl2::{render::Canvas, video::Window};

use std::rc::Rc;

use crate::entity::direction::Direction;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};

pub mod animated;
pub mod blinking;
pub mod directional;
pub mod sprite;

pub fn get_atlas_tile(atlas: &Rc<SpriteAtlas>, name: &str) -> AtlasTile {
    SpriteAtlas::get_tile(atlas, name).unwrap_or_else(|| panic!("Could not find tile {}", name))
}
