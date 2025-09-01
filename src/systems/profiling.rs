use bevy_ecs::system::IntoSystem;
use bevy_ecs::{resource::Resource, system::System};
use circular_buffer::CircularBuffer;
use micromap::Map;
use num_width::NumberWidth;
use parking_lot::{Mutex, RwLock};
use smallvec::SmallVec;
use std::fmt::Display;
use std::time::Duration;
use strum::EnumCount;
use strum_macros::{EnumCount, IntoStaticStr};
use thousands::Separable;

/// The maximum number of systems that can be profiled. Must not be exceeded, or it will panic.
const MAX_SYSTEMS: usize = SystemId::COUNT;
/// The number of durations to keep in the circular buffer.
const TIMING_WINDOW_SIZE: usize = 30;

#[derive(EnumCount, IntoStaticStr, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum SystemId {
    Input,
    PlayerControls,
    Ghost,
    Movement,
    Audio,
    Blinking,
    DirectionalRender,
    DirtyRender,
    HudRender,
    Render,
    DebugRender,
    Present,
    Collision,
    Item,
    PlayerMovement,
    GhostCollision,
    Stage,
    GhostStateAnimation,
    EatenGhost,
}

impl Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self).to_ascii_lowercase())
    }
}

#[derive(Resource, Default, Debug)]
pub struct SystemTimings {
    /// Map of system names to a queue of durations, using a circular buffer.
    ///
    /// Uses a RwLock to allow multiple readers for the HashMap, and a Mutex on the circular buffer for exclusive access.
    /// This is probably overkill, but it's fun to play with.
    ///
    /// Also, we use a micromap::Map as the number of systems is generally quite small.
    /// Just make sure to set the capacity appropriately, or it will panic.
    pub timings: RwLock<Map<SystemId, Mutex<CircularBuffer<TIMING_WINDOW_SIZE, Duration>>, MAX_SYSTEMS>>,
}

impl SystemTimings {
    pub fn add_timing(&self, id: SystemId, duration: Duration) {
        // acquire a upgradable read lock
        let mut timings = self.timings.upgradable_read();

        // happy path, the name is already in the map (no need to mutate the hashmap)
        if timings.contains_key(&id) {
            let queue = timings
                .get(&id)
                .expect("System name not found in map after contains_key check");
            let mut queue = queue.lock();

            queue.push_back(duration);
            return;
        }

        // otherwise, acquire a write lock and insert a new queue
        timings.with_upgraded(|timings| {
            let queue = timings.entry(id).or_insert_with(|| Mutex::new(CircularBuffer::new()));
            queue.lock().push_back(duration);
        });
    }

    pub fn get_stats(&self) -> Map<SystemId, (Duration, Duration), MAX_SYSTEMS> {
        let timings = self.timings.read();
        let mut stats = Map::new();

        for (id, queue) in timings.iter() {
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
                *id,
                (
                    Duration::from_secs_f64(mean / 1000.0),
                    Duration::from_secs_f64(std_dev / 1000.0),
                ),
            );
        }

        stats
    }

    pub fn get_total_stats(&self) -> (Duration, Duration) {
        let duration_sums = {
            let timings = self.timings.read();
            timings
                .iter()
                .map(|(_, queue)| queue.lock().iter().sum::<Duration>())
                .collect::<Vec<_>>()
        };

        let mean = duration_sums.iter().sum::<Duration>() / duration_sums.len() as u32;
        let variance = duration_sums
            .iter()
            .map(|x| {
                let diff_secs = x.as_secs_f64() - mean.as_secs_f64();
                diff_secs * diff_secs
            })
            .sum::<f64>()
            / duration_sums.len() as f64;
        let std_dev_secs = variance.sqrt();

        (mean, Duration::from_secs_f64(std_dev_secs))
    }

    pub fn format_timing_display(&self) -> SmallVec<[String; SystemId::COUNT]> {
        let stats = self.get_stats();
        let (total_avg, total_std) = self.get_total_stats();

        let effective_fps = match 1.0 / total_avg.as_secs_f64() {
            f if f > 100.0 => (f as u32).separate_with_commas(),
            f if f < 10.0 => format!("{:.1} FPS", f),
            f => format!("{:.0} FPS", f),
        };

        // Collect timing data for formatting
        let mut timing_data = vec![(effective_fps, total_avg, total_std)];

        // Sort the stats by average duration
        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        // Add the top 5 most expensive systems
        for (name, (avg, std_dev)) in sorted_stats.iter().take(7) {
            timing_data.push((name.to_string(), *avg, *std_dev));
        }

        // Use the formatting module to format the data
        format_timing_display(timing_data)
    }
}

pub fn profile<S, M>(id: SystemId, system: S) -> impl FnMut(&mut bevy_ecs::world::World)
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
            timings.add_timing(id, duration);
        }
    }
}

// Helper to split a duration into a integer, decimal, and unit
fn get_value(duration: &Duration) -> (u64, u32, &'static str) {
    let (int, decimal, unit) = match duration {
        // if greater than 1 second, return as seconds
        n if n >= &Duration::from_secs(1) => {
            let secs = n.as_secs();
            let decimal = n.as_millis() as u64 % 1000;
            (secs, decimal as u32, "s")
        }
        // if greater than 1 millisecond, return as milliseconds
        n if n >= &Duration::from_millis(1) => {
            let ms = n.as_millis() as u64;
            let decimal = n.as_micros() as u64 % 1000;
            (ms, decimal as u32, "ms")
        }
        // if greater than 1 microsecond, return as microseconds
        n if n >= &Duration::from_micros(1) => {
            let us = n.as_micros() as u64;
            let decimal = n.as_nanos() as u64 % 1000;
            (us, decimal as u32, "µs")
        }
        // otherwise, return as nanoseconds
        n => {
            let ns = n.as_nanos() as u64;
            (ns, 0, "ns")
        }
    };

    (int, decimal, unit)
}

/// Formats timing data into a vector of strings with proper alignment
pub fn format_timing_display(
    timing_data: impl IntoIterator<Item = (String, Duration, Duration)>,
) -> SmallVec<[String; SystemId::COUNT]> {
    let mut iter = timing_data.into_iter().peekable();
    if iter.peek().is_none() {
        return SmallVec::new();
    }

    struct Entry {
        name: String,
        avg_int: u64,
        avg_decimal: u32,
        avg_unit: &'static str,
        std_int: u64,
        std_decimal: u32,
        std_unit: &'static str,
    }

    let entries = iter
        .map(|(name, avg, std_dev)| {
            let (avg_int, avg_decimal, avg_unit) = get_value(&avg);
            let (std_int, std_decimal, std_unit) = get_value(&std_dev);

            Entry {
                name: name.clone(),
                avg_int,
                avg_decimal,
                avg_unit,
                std_int,
                std_decimal,
                std_unit,
            }
        })
        .collect::<SmallVec<[Entry; 12]>>();

    let (max_name_width, max_avg_int_width, max_avg_decimal_width, max_std_int_width, max_std_decimal_width) = entries
        .iter()
        .fold((0, 0, 3, 0, 3), |(name_w, avg_int_w, avg_dec_w, std_int_w, std_dec_w), e| {
            (
                name_w.max(e.name.len()),
                avg_int_w.max(e.avg_int.width() as usize),
                avg_dec_w.max(e.avg_decimal.width() as usize),
                std_int_w.max(e.std_int.width() as usize),
                std_dec_w.max(e.std_decimal.width() as usize),
            )
        });

    entries.iter().map(|e| {
            format!(
                "{name:max_name_width$} : {avg_int:max_avg_int_width$}.{avg_decimal:<max_avg_decimal_width$}{avg_unit} ± {std_int:max_std_int_width$}.{std_decimal:<max_std_decimal_width$}{std_unit}",
                // Content
                name = e.name,
                avg_int = e.avg_int,
                avg_decimal = e.avg_decimal,
                std_int = e.std_int,
                std_decimal = e.std_decimal,
                // Units
                avg_unit = e.avg_unit,
                std_unit = e.std_unit,
                // Padding
                max_name_width = max_name_width,
                max_avg_int_width = max_avg_int_width,
                max_avg_decimal_width = max_avg_decimal_width,
                max_std_int_width = max_std_int_width,
                max_std_decimal_width = max_std_decimal_width
            )
        }).collect::<SmallVec<[String; SystemId::COUNT]>>()
}
