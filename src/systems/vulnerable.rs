use bevy_ecs::{
    query::With,
    system::{Commands, Query, Res},
};

use crate::constants::animation::FRIGHTENED_FLASH_START_TICKS;
use crate::systems::{Ghost, GhostAnimations, GhostCollider, Vulnerable};

/// System that decrements the remaining_ticks on Vulnerable components and removes them when they reach zero
pub fn vulnerable_tick_system(
    mut commands: Commands,
    animations: Res<GhostAnimations>,
    mut vulnerable_query: Query<(bevy_ecs::entity::Entity, &mut Vulnerable, &Ghost), With<GhostCollider>>,
) {
    for (entity, mut vulnerable, ghost_type) in vulnerable_query.iter_mut() {
        if vulnerable.remaining_ticks > 0 {
            vulnerable.remaining_ticks -= 1;
        }

        // When 2 seconds are remaining, start flashing
        if vulnerable.remaining_ticks == FRIGHTENED_FLASH_START_TICKS {
            if let Some(animation_set) = animations.0.get(ghost_type) {
                if let Some(animation) = animation_set.frightened_flashing() {
                    commands.entity(entity).insert(animation.clone());
                }
            }
        }

        if vulnerable.remaining_ticks == 0 {
            commands.entity(entity).remove::<Vulnerable>();
        }
    }
}
