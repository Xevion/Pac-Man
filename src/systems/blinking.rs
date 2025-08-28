use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{Has, With},
    system::{Commands, Query, Res},
};

use crate::systems::{
    components::{DeltaTime, Renderable},
    Hidden,
};

#[derive(Component)]
pub struct Blinking {
    pub timer: f32,
    pub interval: f32,
}

/// Updates blinking entities by toggling their visibility at regular intervals.
///
/// This system manages entities that have both `Blinking` and `Renderable` components,
/// accumulating time and toggling visibility when the specified interval is reached.
pub fn blinking_system(
    mut commands: Commands,
    time: Res<DeltaTime>,
    mut query: Query<(Entity, &mut Blinking, Has<Hidden>), With<Renderable>>,
) {
    for (entity, mut blinking, hidden) in query.iter_mut() {
        blinking.timer += time.0;

        if blinking.timer >= blinking.interval {
            blinking.timer = 0.0;

            // Add or remove the Visible component based on whether it is currently in the query
            if hidden {
                commands.entity(entity).remove::<Hidden>();
            } else {
                commands.entity(entity).insert(Hidden);
            }
        }
    }
}
