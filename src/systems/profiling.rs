use bevy_ecs::prelude::Resource;
use bevy_ecs::system::{IntoSystem, System};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Resource, Default, Debug)]
pub struct SystemTimings {
    pub timings: Mutex<HashMap<&'static str, Duration>>,
}

pub fn profile<S, M>(name: &'static str, system: S) -> impl FnMut(&mut bevy_ecs::world::World)
where
    S: IntoSystem<(), (), M> + 'static,
{
    let mut system: S::System = IntoSystem::into_system(system);
    let mut is_initialized = false;
    move |world: &mut bevy_ecs::world::World| {
        if !is_initialized {
            system.initialize(world);
            is_initialized = true;
        }

        let start = std::time::Instant::now();
        system.run((), world);
        let duration = start.elapsed();

        if let Some(mut timings) = world.get_resource_mut::<SystemTimings>() {
            timings.timings.lock().insert(name, duration);
        }
    }
}
