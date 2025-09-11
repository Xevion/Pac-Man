use bevy_ecs::system::IntoSystem;
use bevy_ecs::{resource::Resource, system::System};
use circular_buffer::CircularBuffer;
use num_width::NumberWidth;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount, EnumIter, IntoStaticStr};
use thousands::Separable;

/// The maximum number of systems that can be profiled. Must not be exceeded, or it will panic.
const MAX_SYSTEMS: usize = SystemId::COUNT;
/// The number of durations to keep in the circular buffer.
const TIMING_WINDOW_SIZE: usize = 30;

/// A timing buffer that tracks durations and automatically inserts zero durations for skipped ticks.
#[derive(Debug, Default)]
pub struct TimingBuffer {
    /// Circular buffer storing timing durations
    buffer: CircularBuffer<TIMING_WINDOW_SIZE, Duration>,
    /// The last tick when this buffer was updated
    last_tick: u64,
}

impl TimingBuffer {
    /// Adds a timing duration for the current tick.
    ///
    /// # Panics
    ///
    /// Panics if `current_tick` is less than `last_tick`, indicating time went backwards.
    pub fn add_timing(&mut self, duration: Duration, current_tick: u64) {
        if current_tick < self.last_tick {
            panic!(
                "Time went backwards: current_tick ({}) < last_tick ({})",
                current_tick, self.last_tick
            );
        }

        // Insert zero durations for any skipped ticks (but not the current tick)
        if current_tick > self.last_tick {
            let skipped_ticks = current_tick - self.last_tick - 1;
            for _ in 0..skipped_ticks {
                self.buffer.push_back(Duration::ZERO);
            }
        }

        // Add the actual timing
        self.buffer.push_back(duration);
        self.last_tick = current_tick;
    }

    /// Gets the most recent timing from the buffer.
    pub fn get_most_recent_timing(&self) -> Duration {
        self.buffer.back().copied().unwrap_or(Duration::ZERO)
    }

    /// Gets statistics for this timing buffer.
    ///
    /// # Panics
    ///
    /// Panics if `current_tick` is less than `last_tick`, indicating time went backwards.
    pub fn get_stats(&mut self, current_tick: u64) -> (Duration, Duration) {
        // Insert zero durations for any skipped ticks since last update (but not the current tick)
        if current_tick > self.last_tick {
            let skipped_ticks = current_tick - self.last_tick - 1;
            for _ in 0..skipped_ticks {
                self.buffer.push_back(Duration::ZERO);
            }
            self.last_tick = current_tick;
        }

        // Calculate statistics using Welford's algorithm
        let mut sample_count = 0u16;
        let mut running_mean = 0.0;
        let mut sum_squared_diff = 0.0;

        let skip = self.last_tick.saturating_sub(current_tick);
        for duration in self.buffer.iter().skip(skip as usize) {
            let duration_secs = duration.as_secs_f32();
            sample_count += 1;

            let diff_from_mean = duration_secs - running_mean;
            running_mean += diff_from_mean / sample_count as f32;

            let diff_from_new_mean = duration_secs - running_mean;
            sum_squared_diff += diff_from_mean * diff_from_new_mean;
        }

        if sample_count > 0 {
            let variance = if sample_count > 1 {
                sum_squared_diff / (sample_count - 1) as f32
            } else {
                0.0
            };

            (
                Duration::from_secs_f32(running_mean),
                Duration::from_secs_f32(variance.sqrt()),
            )
        } else {
            (Duration::ZERO, Duration::ZERO)
        }
    }
}

/// A resource that tracks the current game tick using an atomic counter.
/// This ensures thread-safe access to the tick counter across systems.
#[derive(Resource, Debug)]
pub struct Timing {
    /// Atomic counter for the current game tick
    current_tick: AtomicU64,
}

impl Timing {
    /// Creates a new Timing resource starting at tick 0
    pub fn new() -> Self {
        Self {
            current_tick: AtomicU64::new(0),
        }
    }

    /// Gets the current tick value
    pub fn get_current_tick(&self) -> u64 {
        self.current_tick.load(Ordering::Relaxed)
    }

    /// Increments the tick counter and returns the new value
    pub fn increment_tick(&self) -> u64 {
        self.current_tick.fetch_add(1, Ordering::Relaxed) + 1
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(EnumCount, EnumIter, IntoStaticStr, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum SystemId {
    Total,
    Input,
    PlayerControls,
    Ghost,
    Movement,
    Audio,
    Blinking,
    DirectionalRender,
    LinearRender,
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
    TimeToLive,
    PauseManager,
}

impl Display for SystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use strum_macros::IntoStaticStr to get the static string
        write!(f, "{}", Into::<&'static str>::into(self).to_ascii_lowercase())
    }
}

#[derive(Resource, Debug)]
pub struct SystemTimings {
    /// Statically sized map of system names to timing buffers.
    pub timings: micromap::Map<SystemId, Mutex<TimingBuffer>, MAX_SYSTEMS>,
}

impl Default for SystemTimings {
    fn default() -> Self {
        let mut timings = micromap::Map::new();

        // Pre-populate with all SystemId variants to avoid runtime allocations
        for id in SystemId::iter() {
            timings.insert(id, Mutex::new(TimingBuffer::default()));
        }

        Self { timings }
    }
}

impl SystemTimings {
    pub fn add_timing(&self, id: SystemId, duration: Duration, current_tick: u64) {
        // Since all SystemId variants are pre-populated, we can use a simple read lock
        let buffer = self
            .timings
            .get(&id)
            .expect("SystemId not found in pre-populated map - this is a bug");
        buffer.lock().add_timing(duration, current_tick);
    }

    /// Add timing for the Total system (total frame time including scheduler.run)
    pub fn add_total_timing(&self, duration: Duration, current_tick: u64) {
        self.add_timing(SystemId::Total, duration, current_tick);
    }

    pub fn get_stats(&self, current_tick: u64) -> micromap::Map<SystemId, (Duration, Duration), MAX_SYSTEMS> {
        let mut stats = micromap::Map::new();

        // Iterate over all SystemId variants to ensure every system has an entry
        for id in SystemId::iter() {
            let buffer = self
                .timings
                .get(&id)
                .expect("SystemId not found in pre-populated map - this is a bug");

            let (average, standard_deviation) = buffer.lock().get_stats(current_tick);
            stats.insert(id, (average, standard_deviation));
        }

        stats
    }

    pub fn format_timing_display(&self, current_tick: u64) -> SmallVec<[String; SystemId::COUNT]> {
        let stats = self.get_stats(current_tick);

        // Get the Total system metrics instead of averaging all systems
        let (total_avg, total_std) = stats
            .get(&SystemId::Total)
            .copied()
            .unwrap_or((Duration::ZERO, Duration::ZERO));

        let effective_fps = match 1.0 / total_avg.as_secs_f64() {
            f if f > 100.0 => format!("{:>5} FPS", (f as u32).separate_with_commas()),
            f if f < 10.0 => format!("{:.1} FPS", f),
            f => format!("{:5.0} FPS", f),
        };

        // Collect timing data for formatting
        let mut timing_data = vec![(effective_fps, total_avg, total_std)];

        // Sort the stats by average duration, excluding the Total system
        let mut sorted_stats: Vec<_> = stats.iter().filter(|(id, _)| **id != SystemId::Total).collect();
        sorted_stats.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        // Add the top 7 most expensive systems (excluding Total)
        for (name, (avg, std_dev)) in sorted_stats.iter().take(9) {
            timing_data.push((name.to_string(), *avg, *std_dev));
        }

        // Use the formatting module to format the data
        format_timing_display(timing_data)
    }

    /// Returns a list of systems with their timings, likely responsible for slow frame timings.
    ///
    /// First, checks if any systems took longer than 2ms on the most recent tick.
    /// If none exceed 2ms, accumulates systems until the top 30% of total timing
    /// is reached, stopping at 5 systems maximum.
    ///
    /// Returns tuples of (SystemId, Duration) in a SmallVec capped at 5 items.
    pub fn get_slowest_systems(&self) -> SmallVec<[(SystemId, Duration); 5]> {
        let mut system_timings: Vec<(SystemId, Duration)> = Vec::new();
        let mut total_duration = Duration::ZERO;

        // Collect most recent timing for each system (excluding Total)
        for id in SystemId::iter() {
            if id == SystemId::Total {
                continue;
            }

            if let Some(buffer) = self.timings.get(&id) {
                let recent_timing = buffer.lock().get_most_recent_timing();
                system_timings.push((id, recent_timing));
                total_duration += recent_timing;
            }
        }

        // Sort by duration (highest first)
        system_timings.sort_by(|a, b| b.1.cmp(&a.1));

        // Check for systems over 2ms threshold
        let over_threshold: SmallVec<[(SystemId, Duration); 5]> = system_timings
            .iter()
            .filter(|(_, duration)| duration.as_millis() >= 2)
            .copied()
            .collect();

        if !over_threshold.is_empty() {
            return over_threshold;
        }

        // Accumulate top systems until 30% of total is reached (max 5 systems)
        let threshold = total_duration.as_nanos() as f64 * 0.3;
        let mut accumulated = 0u128;
        let mut result = SmallVec::new();

        for (id, duration) in system_timings.iter().take(5) {
            result.push((*id, *duration));
            accumulated += duration.as_nanos();

            if accumulated as f64 >= threshold {
                break;
            }
        }

        result
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

        if let (Some(timings), Some(timing)) = (world.get_resource::<SystemTimings>(), world.get_resource::<Timing>()) {
            let current_tick = timing.get_current_tick();
            timings.add_timing(id, duration, current_tick);
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

    let (max_avg_int_width, max_avg_decimal_width, max_std_int_width, max_std_decimal_width) =
        entries
            .iter()
            .fold((0, 3, 0, 3), |(avg_int_w, avg_dec_w, std_int_w, std_dec_w), e| {
                (
                    avg_int_w.max(e.avg_int.width() as usize),
                    avg_dec_w.max(e.avg_decimal.width() as usize),
                    std_int_w.max(e.std_int.width() as usize),
                    std_dec_w.max(e.std_decimal.width() as usize),
                )
            });

    let max_name_width = SystemId::iter()
        .map(|id| id.to_string().len())
        .max()
        .expect("SystemId::iter() returned an empty iterator");

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
