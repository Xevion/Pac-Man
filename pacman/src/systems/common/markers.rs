//! Entity-class marker components. Each marker carries, via `#[require]`, the
//! constant components shared by every entity of its class; spawn sites supply
//! only the per-entity variable components (position, sprite, animation, and the
//! identity/state that differs between instances).

use bevy_ecs::component::Component;

use crate::constants;
use crate::map::direction::Direction;
use crate::systems::collision::{Collider, GhostCollider, ItemCollider, PacmanCollider};
use crate::systems::common::components::{EntityType, MovementModifiers};
use crate::systems::ghost::{GhostAnimationState, GhostTarget, LastAnimationState};
use crate::systems::movement::{BufferedDirection, Velocity};
use crate::systems::player::PlayerControlled;

/// The player entity. Spawn alongside `Position`, `Renderable`, and a
/// `DirectionalAnimation`.
#[derive(Component)]
#[require(
    PlayerControlled,
    Velocity = Velocity { speed: constants::mechanics::PLAYER_SPEED, direction: Direction::Left },
    MovementModifiers,
    BufferedDirection = BufferedDirection::None,
    EntityType = EntityType::Player,
    Collider = Collider { size: constants::collider::PLAYER_SIZE },
    PacmanCollider = PacmanCollider,
)]
pub struct Pacman;

/// A ghost entity. Spawn alongside `GhostType`, `Position`, `Velocity`,
/// `Renderable`, a `DirectionalAnimation`, and the initial `GhostState`.
#[derive(Component)]
#[require(
    EntityType = EntityType::Ghost,
    Collider = Collider { size: constants::collider::GHOST_SIZE },
    GhostCollider = GhostCollider,
    GhostTarget,
    LastAnimationState = LastAnimationState(GhostAnimationState::Normal),
)]
pub struct Ghost;

/// A collectible item (pellet, power pellet, or fruit). Spawn alongside
/// `Position`, `Renderable`, an `EntityType`, and a `Collider`.
#[derive(Component)]
#[require(ItemCollider = ItemCollider)]
pub struct Item;
