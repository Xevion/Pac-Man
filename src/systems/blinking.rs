use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{Has, With},
    system::{Commands, Query, Res},
};

use crate::systems::{
    components::{DeltaTime, Renderable},
    Frozen, Hidden,
};

#[derive(Component, Debug)]
pub struct Blinking {
    pub timer: f32,
    pub interval: f32,
}

impl Blinking {
    pub fn new(interval: f32) -> Self {
        Self { timer: 0.0, interval }
    }
}

/// Updates blinking entities by toggling their visibility at regular intervals.
///
/// This system manages entities that have both `Blinking` and `Renderable` components,
/// accumulating time and toggling visibility when the specified interval is reached.
pub fn blinking_system(
    mut commands: Commands,
    time: Res<DeltaTime>,
    mut query: Query<(Entity, &mut Blinking, Has<Hidden>, Has<Frozen>), With<Renderable>>,
) {
    for (entity, mut blinking, hidden, frozen) in query.iter_mut() {
        // If the entity is frozen, blinking is disabled and the entity is unhidden (if it was hidden)
        if frozen {
            if hidden {
                commands.entity(entity).remove::<Hidden>();
            }

            continue;
        }

        // Increase the timer by the delta time
        blinking.timer += time.0;

        // If the timer is less than the interval, there's nothing to do yet
        if blinking.timer < blinking.interval {
            continue;
        }

        // Subtract the interval (allows for the timer to retain partial interval progress)
        blinking.timer -= blinking.interval;

        // Toggle the Hidden component
        if hidden {
            commands.entity(entity).remove::<Hidden>();
        } else {
            commands.entity(entity).insert(Hidden);
        }
    }
}
