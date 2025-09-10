use bevy_ecs::{
    component::Component,
    query::{Has, With},
    system::{Query, Res},
};

use crate::systems::{DeltaTime, Frozen, Renderable, Visibility};

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
pub fn blinking_system(time: Res<DeltaTime>, mut query: Query<(&mut Blinking, &mut Visibility, Has<Frozen>), With<Renderable>>) {
    for (mut blinking, mut visibility, frozen) in query.iter_mut() {
        // If the entity is frozen, blinking is disabled and the entity is made visible
        if frozen {
            visibility.show();
            continue;
        }

        // Increase the timer by the delta ticks
        blinking.tick_timer += time.ticks;

        // Handle zero interval case (immediate toggling)
        if blinking.interval_ticks == 0 {
            if time.ticks > 0 {
                visibility.toggle();
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

        // Toggle the visibility for each complete interval
        // Since toggling twice is a no-op, we only need to toggle if the count is odd
        if complete_intervals % 2 == 1 {
            visibility.toggle();
        }
    }
}
