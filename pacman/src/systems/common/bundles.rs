use bevy_ecs::bundle::Bundle;

use crate::systems::animation::DirectionalAnimation;
use crate::systems::collision::{Collider, GhostCollider, ItemCollider, PacmanCollider};
use crate::systems::common::{EntityType, MovementModifiers};
use crate::systems::ghost::{GhostState, GhostTarget, GhostType, LastAnimationState};
use crate::systems::movement::{BufferedDirection, Position, Velocity};
use crate::systems::player::PlayerControlled;
use crate::systems::render::Renderable;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub velocity: Velocity,
    pub buffered_direction: BufferedDirection,
    pub sprite: Renderable,
    pub directional_animation: DirectionalAnimation,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub movement_modifiers: MovementModifiers,
    pub pacman_collider: PacmanCollider,
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub position: Position,
    pub sprite: Renderable,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub item_collider: ItemCollider,
}

#[derive(Bundle)]
pub struct GhostBundle {
    pub ghost: GhostType,
    pub position: Position,
    pub velocity: Velocity,
    pub sprite: Renderable,
    pub directional_animation: DirectionalAnimation,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub ghost_collider: GhostCollider,
    pub ghost_state: GhostState,
    pub last_animation_state: LastAnimationState,
    pub ghost_target: GhostTarget,
}
