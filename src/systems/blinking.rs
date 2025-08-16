use bevy_ecs::{
    component::Component,
    system::{Query, Res},
};

use crate::systems::components::{DeltaTime, Renderable};

#[derive(Component)]
pub struct Blinking {
    pub timer: f32,
    pub interval: f32,
}

/// Updates blinking entities by toggling their visibility at regular intervals.
///
/// This system manages entities that have both `Blinking` and `Renderable` components,
/// accumulating time and toggling visibility when the specified interval is reached.
pub fn blinking_system(time: Res<DeltaTime>, mut query: Query<(&mut Blinking, &mut Renderable)>) {
    for (mut blinking, mut renderable) in query.iter_mut() {
        blinking.timer += time.0;

        if blinking.timer >= blinking.interval {
            blinking.timer = 0.0;
            renderable.visible = !renderable.visible;
        }
    }
}
