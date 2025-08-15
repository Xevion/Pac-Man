use bevy_ecs::prelude::Resource;
use bevy_ecs::system::{IntoSystem, System};
use micromap::Map;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::time::Duration;

const TIMING_WINDOW_SIZE: usize = 90; // 1.5 seconds at 60 FPS

#[derive(Resource, Default, Debug)]
pub struct SystemTimings {
    pub timings: Mutex<Map<&'static str, VecDeque<Duration>, 15>>,
}

impl SystemTimings {
    pub fn add_timing(&self, name: &'static str, duration: Duration) {
        let mut timings = self.timings.lock();
        let queue = timings.entry(name).or_insert_with(VecDeque::new);

        queue.push_back(duration);
        if queue.len() > TIMING_WINDOW_SIZE {
            queue.pop_front();
        }
    }

    pub fn get_stats(&self) -> Map<&'static str, (Duration, Duration), 10> {
        let timings = self.timings.lock();
        let mut stats = Map::new();

        for (name, queue) in timings.iter() {
            if queue.is_empty() {
                continue;
            }

            let durations: Vec<f64> = queue.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
            let count = durations.len() as f64;

            let sum: f64 = durations.iter().sum();
            let mean = sum / count;

            let variance = durations.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            stats.insert(
                *name,
                (
                    Duration::from_secs_f64(mean / 1000.0),
                    Duration::from_secs_f64(std_dev / 1000.0),
                ),
            );
        }

        stats
    }

    pub fn get_total_stats(&self) -> (Duration, Duration) {
        let timings = self.timings.lock();
        let mut all_durations = Vec::new();

        for queue in timings.values() {
            all_durations.extend(queue.iter().map(|d| d.as_secs_f64() * 1000.0));
        }

        if all_durations.is_empty() {
            return (Duration::ZERO, Duration::ZERO);
        }

        let count = all_durations.len() as f64;
        let sum: f64 = all_durations.iter().sum();
        let mean = sum / count;

        let variance = all_durations.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        (
            Duration::from_secs_f64(mean / 1000.0),
            Duration::from_secs_f64(std_dev / 1000.0),
        )
    }
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

        if let Some(timings) = world.get_resource::<SystemTimings>() {
            timings.add_timing(name, duration);
        }
    }
}
