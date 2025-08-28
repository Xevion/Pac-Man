//! The Entity-Component-System (ECS) module.
//!
//! This module contains all the ECS-related logic, including components, systems,
//! and resources.

pub mod audio;
pub mod blinking;
pub mod collision;
pub mod components;
pub mod debug;
pub mod formatting;
pub mod ghost;
pub mod input;
pub mod item;
pub mod movement;
pub mod player;
pub mod profiling;
pub mod render;
pub mod stage;
pub mod vulnerable;

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
pub use self::vulnerable::*;
