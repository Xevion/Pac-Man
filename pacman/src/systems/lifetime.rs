use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};

use crate::systems::DeltaTime;

/// Component for entities that should be automatically deleted after a certain number of ticks
#[derive(Component, Debug, Clone, Copy)]
pub struct TimeToLive {
    pub remaining_ticks: u32,
}

impl TimeToLive {
    pub fn new(ticks: u32) -> Self {
        Self { remaining_ticks: ticks }
    }
}

/// System that manages entities with TimeToLive components, decrementing their remaining ticks
/// and despawning them when they expire
pub fn time_to_live_system(mut commands: Commands, dt: Res<DeltaTime>, mut query: Query<(Entity, &mut TimeToLive)>) {
    for (entity, mut ttl) in query.iter_mut() {
        if ttl.remaining_ticks <= dt.ticks {
            // Entity has expired, despawn it
            commands.entity(entity).despawn();
        } else {
            // Decrement remaining time
            ttl.remaining_ticks = ttl.remaining_ticks.saturating_sub(dt.ticks);
        }
    }
}
