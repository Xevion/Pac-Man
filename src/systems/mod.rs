//! The Entity-Component-System (ECS) module.
//!
//! This module contains all the ECS-related logic, including components, systems,
//! and resources.

#[cfg_attr(coverage_nightly, coverage(off))]
pub mod audio;
pub mod blinking;
pub mod collision;
pub mod components;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod debug;
pub mod ghost;
pub mod input;
pub mod item;
pub mod movement;
pub mod player;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod profiling;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod render;
pub mod stage;

pub use self::audio::*;
pub use self::blinking::*;
pub use self::collision::*;
pub use self::components::*;
pub use self::debug::*;
pub use self::ghost::*;
pub use self::input::*;
pub use self::item::*;
pub use self::movement::*;
pub use self::player::*;
pub use self::profiling::*;
pub use self::render::*;
pub use self::stage::*;
