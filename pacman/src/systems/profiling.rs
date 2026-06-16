use bevy_ecs::system::IntoSystem;
use bevy_ecs::{resource::Resource, system::System};
use circular_buffer::CircularBuffer;
use num_width::NumberWidth;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thousands::Separable;

/// Inline capacity for the timing-display rows: the FPS line plus the few slowest
/// systems. Sized so the overlay never spills to the heap in practice.
const TIMING_ROWS: usize = 16;
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

/// Label for the whole-frame timing row (the total schedule run), distinguished from
/// the per-system rows in both the overlay and the slow-frame report.
pub const TOTAL: &str = "total";

/// Per-label timing buffers, keyed by the `&'static str` label passed to [`profile`].
///
/// Labels register lazily on first sighting, so the set of profiled systems lives at
/// the `profile()` call sites rather than in a central registry that must be kept in
/// sync. A single outer mutex is sufficient: profiled systems are wrapped as exclusive
/// systems and so run sequentially, never contending.
#[derive(Resource, Default, Debug)]
pub struct SystemTimings {
    timings: Mutex<HashMap<&'static str, TimingBuffer>>,
}

impl SystemTimings {
    /// Records a timing sample for `label`, creating its buffer on first sighting.
    pub fn add_timing(&self, label: &'static str, duration: Duration, current_tick: u64) {
        self.timings
            .lock()
            .entry(label)
            .or_default()
            .add_timing(duration, current_tick);
    }

    /// Records the whole-frame timing (total schedule run) under the [`TOTAL`] label.
    pub fn add_total_timing(&self, duration: Duration, current_tick: u64) {
        self.add_timing(TOTAL, duration, current_tick);
    }

    /// Computes (mean, standard deviation) per label over the current window.
    pub fn get_stats(&self, current_tick: u64) -> HashMap<&'static str, (Duration, Duration)> {
        self.timings
            .lock()
            .iter_mut()
            .map(|(label, buffer)| (*label, buffer.get_stats(current_tick)))
            .collect()
    }

    pub fn format_timing_display(&self, current_tick: u64) -> SmallVec<[String; TIMING_ROWS]> {
        let stats = self.get_stats(current_tick);

        // Use the Total row for the headline FPS rather than averaging all systems.
        let (total_avg, total_std) = stats.get(TOTAL).copied().unwrap_or((Duration::ZERO, Duration::ZERO));

        let effective_fps = match 1.0 / total_avg.as_secs_f64() {
            f if f > 100.0 => format!("{:>5} FPS", (f as u32).separate_with_commas()),
            f if f < 10.0 => format!("{:.1} FPS", f),
            f => format!("{:5.0} FPS", f),
        };

        let mut timing_data = vec![(effective_fps, total_avg, total_std)];

        // Sort by average duration descending, excluding Total. Tie-break on the label so
        // equal timings keep a stable order frame-to-frame (HashMap iteration is arbitrary).
        let mut sorted_stats: Vec<(&'static str, (Duration, Duration))> =
            stats.into_iter().filter(|(label, _)| *label != TOTAL).collect();
        sorted_stats.sort_by_key(|(label, (avg, _))| (std::cmp::Reverse(*avg), *label));

        for (label, (avg, std_dev)) in sorted_stats.iter().take(9) {
            timing_data.push((label.to_string(), *avg, *std_dev));
        }

        format_timing_display(timing_data)
    }

    /// Returns the systems likely responsible for a slow frame.
    ///
    /// First, checks if any systems took longer than 2ms on the most recent tick.
    /// If none exceed 2ms, accumulates systems until the top 30% of total timing
    /// is reached, stopping at 5 systems maximum.
    pub fn get_slowest_systems(&self) -> SmallVec<[(&'static str, Duration); 5]> {
        let mut system_timings: Vec<(&'static str, Duration)> = Vec::new();
        let mut total_duration = Duration::ZERO;

        for (label, buffer) in self.timings.lock().iter() {
            if *label == TOTAL {
                continue;
            }
            let recent_timing = buffer.get_most_recent_timing();
            system_timings.push((label, recent_timing));
            total_duration += recent_timing;
        }

        // Sort by duration descending, tie-breaking on the label for a stable order.
        system_timings.sort_by_key(|(label, duration)| (std::cmp::Reverse(*duration), *label));

        // Check for systems over 2ms threshold
        let over_threshold: SmallVec<[(&'static str, Duration); 5]> = system_timings
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

        for (label, duration) in system_timings.iter().take(5) {
            result.push((*label, *duration));
            accumulated += duration.as_nanos();

            if accumulated as f64 >= threshold {
                break;
            }
        }

        result
    }
}

/// Wraps `system` so each run is timed and emitted as a Tracy zone.
///
/// The wrapped system is boxed, so the returned closure captures a pointer rather
/// than the system's full parameter state (which can be hundreds of bytes). The
/// schedule materializes every system into one large nest of tuples on the stack
/// at startup; keeping each element pointer-sized keeps that construction well
/// within Emscripten's small main stack, regardless of how many systems exist.
///
/// The wrapped system is invoked via [`System::run`], which does **not** run Bevy's
/// param validation. The stock scheduler validates params first and silently skips a
/// system whose params aren't satisfiable (e.g. an empty `Single`); that skip does not
/// happen here. A system that can legitimately have unsatisfiable params must opt into
/// tolerance itself -- use `Option<Single>`/`Populated` or an explicit guard rather
/// than a bare `Single`, or it will panic here instead of being skipped.
pub fn profile<S, M>(label: &'static str, system: S) -> impl FnMut(&mut bevy_ecs::world::World)
where
    S: IntoSystem<(), (), M> + 'static,
{
    let mut system: Box<dyn System<In = (), Out = ()>> = Box::new(IntoSystem::into_system(system));
    let mut is_initialized = false;
    let mut warned = false;
    let name = label;
    move |world: &mut bevy_ecs::world::World| {
        if !is_initialized {
            system.initialize(world);
            is_initialized = true;
        }

        let start = std::time::Instant::now();
        {
            let _zone = crate::tracy::zone(name, file!(), line!());
            // Mirror Bevy's stock scheduler: skip a system whose params aren't
            // satisfiable this frame (an empty `Single`, a missing resource, ...) rather
            // than running it. `System::run` -- unlike the executor -- does not validate,
            // so we do it explicitly. Warn once per system so a persistent skip stays
            // visible without spamming a line every frame.
            match system.validate_param(world) {
                Ok(()) => system.run((), world),
                Err(error) => {
                    if !warned {
                        warned = true;
                        tracing::warn!(system = name, "skipping system: parameter validation failed: {error}");
                    }
                }
            }
        }
        let duration = start.elapsed();

        if let (Some(timings), Some(timing)) = (world.get_resource::<SystemTimings>(), world.get_resource::<Timing>()) {
            let current_tick = timing.get_current_tick();
            timings.add_timing(label, duration, current_tick);
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
) -> SmallVec<[String; TIMING_ROWS]> {
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

    // Width the name column to the rows actually being shown so the colons align. The
    // early return above guarantees at least one entry, so `max` is non-empty.
    let max_name_width = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);

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
        }).collect::<SmallVec<[String; TIMING_ROWS]>>()
}
