use std::cell::RefCell;
use std::rc::Rc;

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

pub mod animated;
pub mod blinking;
pub mod directional;
pub mod sprite;
pub mod text;

pub fn get_atlas_tile(atlas: &Rc<RefCell<SpriteAtlas>>, name: &str) -> AtlasTile {
    SpriteAtlas::get_tile(atlas, name).unwrap_or_else(|| panic!("Could not find tile {name}"))
}
