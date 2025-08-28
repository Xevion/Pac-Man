use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query};

use crate::systems::{GhostCollider, Vulnerable};

/// System that decrements the remaining_ticks on Vulnerable components and removes them when they reach zero
pub fn vulnerable_tick_system(
    mut commands: Commands,
    mut vulnerable_query: Query<(bevy_ecs::entity::Entity, &mut Vulnerable), With<GhostCollider>>,
) {
    for (entity, mut vulnerable) in vulnerable_query.iter_mut() {
        if vulnerable.remaining_ticks > 0 {
            vulnerable.remaining_ticks -= 1;
        }

        if vulnerable.remaining_ticks == 0 {
            commands.entity(entity).remove::<Vulnerable>();
        }
    }
}
