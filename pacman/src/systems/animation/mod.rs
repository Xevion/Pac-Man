mod blinking;
mod directional;
mod linear;

pub use self::blinking::*;
pub use self::directional::*;
pub use self::linear::*;

use bevy_ecs::component::Component;

/// Tag component for Pac-Man during his death animation.
/// This is mainly because the Frozen tag would stop both movement and animation,
/// while the Dying tag can signal that the animation should continue despite being frozen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Dying;
