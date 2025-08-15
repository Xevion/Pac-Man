use bevy_ecs::prelude::Resource;
use bevy_ecs::system::{IntoSystem, System};
use circular_buffer::CircularBuffer;
use micromap::Map;
use parking_lot::{Mutex, RwLock};
use std::time::Duration;
use thousands::Separable;

const TIMING_WINDOW_SIZE: usize = 30;

#[derive(Resource, Default, Debug)]
pub struct SystemTimings {
    /// Map of system names to a queue of durations, using a circular buffer.
    ///
    /// Uses a RwLock to allow multiple readers for the HashMap, and a Mutex on the circular buffer for exclusive access.
    /// This is probably overkill, but it's fun to play with.
    ///
    /// Also, we use a micromap::Map as the number of systems is generally quite small.
    /// Just make sure to set the capacity appropriately, or it will panic.
    pub timings: RwLock<Map<&'static str, Mutex<CircularBuffer<TIMING_WINDOW_SIZE, Duration>>, 10>>,
}

impl SystemTimings {
    pub fn add_timing(&self, name: &'static str, duration: Duration) {
        // acquire a upgradable read lock
        let mut timings = self.timings.upgradable_read();

        // happy path, the name is already in the map (no need to mutate the hashmap)
        if timings.contains_key(name) {
            let queue = timings
                .get(name)
                .expect("System name not found in map after contains_key check");
            let mut queue = queue.lock();

            queue.push_back(duration);
            return;
        }

        // otherwise, acquire a write lock and insert a new queue
        timings.with_upgraded(|timings| {
            let queue = timings.entry(name).or_insert_with(|| Mutex::new(CircularBuffer::new()));
            queue.lock().push_back(duration);
        });
    }

    pub fn get_stats(&self) -> Map<&'static str, (Duration, Duration), 10> {
        let timings = self.timings.read();
        let mut stats = Map::new();

        for (name, queue) in timings.iter() {
            if queue.lock().is_empty() {
                continue;
            }

            let durations: Vec<f64> = queue.lock().iter().map(|d| d.as_secs_f64() * 1000.0).collect();
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
        let timings = self.timings.read();
        let mut all_durations = Vec::new();

        for queue in timings.values() {
            all_durations.extend(queue.lock().iter().map(|d| d.as_secs_f64() * 1000.0));
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

    pub fn format_timing_display(&self) -> String {
        let stats = self.get_stats();
        let (total_avg, total_std) = self.get_total_stats();

        let effective_fps = match 1.0 / total_avg.as_secs_f64() {
            f if f > 100.0 => (f as u32).separate_with_commas(),
            f if f < 10.0 => format!("{:.1} FPS", f),
            f => format!("{:.0} FPS", f),
        };

        // Collect timing data for formatting
        let mut timing_data = Vec::new();

        // Add total stats
        timing_data.push((effective_fps, total_avg, total_std));

        // Add top 5 most expensive systems
        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        for (name, (avg, std_dev)) in sorted_stats.iter().take(5) {
            timing_data.push((name.to_string(), *avg, *std_dev));
        }

        // Use the formatting module to format the data
        crate::systems::formatting::format_timing_display(timing_data)
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
