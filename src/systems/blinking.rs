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
    pub tick_timer: u32,
    pub interval_ticks: u32,
}

impl Blinking {
    pub fn new(interval_ticks: u32) -> Self {
        Self {
            tick_timer: 0,
            interval_ticks,
        }
    }
}

/// Updates blinking entities by toggling their visibility at regular intervals.
///
/// This system manages entities that have both `Blinking` and `Renderable` components,
/// accumulating ticks and toggling visibility when the specified interval is reached.
/// Uses integer arithmetic for deterministic behavior.
#[allow(clippy::type_complexity)]
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

        // Increase the timer by the delta ticks
        blinking.tick_timer += time.ticks;

        // Handle zero interval case (immediate toggling)
        if blinking.interval_ticks == 0 {
            if time.ticks > 0 {
                if hidden {
                    commands.entity(entity).remove::<Hidden>();
                } else {
                    commands.entity(entity).insert(Hidden);
                }
            }
            continue;
        }

        // Calculate how many complete intervals have passed
        let complete_intervals = blinking.tick_timer / blinking.interval_ticks;

        // If no complete intervals have passed, there's nothing to do yet
        if complete_intervals == 0 {
            continue;
        }

        // Update the timer to the remainder after complete intervals
        blinking.tick_timer %= blinking.interval_ticks;

        // Toggle the Hidden component for each complete interval
        // Since toggling twice is a no-op, we only need to toggle if the count is odd
        if complete_intervals % 2 == 1 {
            if hidden {
                commands.entity(entity).remove::<Hidden>();
            } else {
                commands.entity(entity).insert(Hidden);
            }
        }
    }
}
