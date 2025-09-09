//! This module contains all the systems in the game.

// These modules are excluded from coverage.
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod audio;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod debug;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod profiling;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod render;

pub mod animation;
pub mod blinking;
pub mod collision;
pub mod common;
pub mod ghost;
pub mod input;
pub mod item;
pub mod lifetime;
pub mod movement;
pub mod player;
pub mod state;

// Re-export all the modules. Do not fine-tune the exports.

pub use self::animation::*;
pub use self::audio::*;
pub use self::blinking::*;
pub use self::collision::*;
pub use self::common::*;
pub use self::debug::*;
pub use self::ghost::*;
pub use self::input::*;
pub use self::item::*;
pub use self::lifetime::*;
pub use self::movement::*;
pub use self::player::*;
pub use self::profiling::*;
pub use self::render::*;
pub use self::state::*;
