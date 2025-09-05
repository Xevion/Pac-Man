//! Custom tracing formatter with tick counter integration

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use time::macros::format_description;
use time::{format_description::FormatItem, OffsetDateTime};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// Global atomic counter for tracking game ticks
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Maximum value for tick counter display (16-bit hex)
const TICK_DISPLAY_MASK: u64 = 0xFFFF;

/// Cached format description for timestamps
/// Uses 3 subsecond digits on Emscripten, 5 otherwise for better performance
#[cfg(target_os = "emscripten")]
const TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second].[subsecond digits:3]");

#[cfg(not(target_os = "emscripten"))]
const TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second].[subsecond digits:5]");

/// A custom formatter that includes both timestamp and tick counter in hexadecimal
///
/// This formatter provides:
/// - High-precision timestamps (HH:MM:SS.mmm on Emscripten, HH:MM:SS.mmmmm otherwise)
/// - Hexadecimal tick counter for frame correlation
/// - Standard log level and target information
///
/// Performance considerations:
/// - Timestamp format is cached at compile time
/// - Tick counter access is atomic and very fast
/// - Combined formatting operations for efficiency
pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
        // Format timestamp using cached format description
        let now = OffsetDateTime::now_utc();
        let formatted_time = now.format(&TIMESTAMP_FORMAT).map_err(|e| {
            // Preserve the original error information for debugging
            eprintln!("Failed to format timestamp: {}", e);
            fmt::Error
        })?;

        // Get tick count and format everything together
        let tick_count = get_tick_count();
        let metadata = event.metadata();

        // Combined formatting: timestamp, tick counter, level, and target in one write
        write!(
            writer,
            "{} 0x{:04X} {:5} {}: ",
            formatted_time,
            tick_count & TICK_DISPLAY_MASK,
            metadata.level(),
            metadata.target()
        )?;

        // Format the fields (the actual log message)
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// Increment the global tick counter by 1
///
/// This should be called once per game tick/frame from the main game loop
pub fn increment_tick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Get the current tick count
///
/// Returns the current value of the global tick counter
pub fn get_tick_count() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Reset the tick counter to 0
///
/// This can be used for testing or when restarting the game
#[allow(dead_code)]
pub fn reset_tick_counter() {
    TICK_COUNTER.store(0, Ordering::Relaxed);
}
