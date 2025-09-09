//! Custom tracing formatter with tick counter integration

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use time::macros::format_description;
use time::{format_description::FormatItem, OffsetDateTime};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
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
/// Re-implementation of the Full formatter to add a tick counter and timestamp.
pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
        let meta = event.metadata();

        // 1) Timestamp (dimmed when ANSI)
        let now = OffsetDateTime::now_utc();
        let formatted_time = now.format(&TIMESTAMP_FORMAT).map_err(|e| {
            eprintln!("Failed to format timestamp: {}", e);
            fmt::Error
        })?;
        write_dimmed(&mut writer, formatted_time)?;
        writer.write_char(' ')?;

        // 2) Tick counter, dim when ANSI
        let tick_count = get_tick_count() & TICK_DISPLAY_MASK;
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m0x{:04X}\x1b[0m ", tick_count)?;
        } else {
            write!(writer, "0x{:04X} ", tick_count)?;
        }

        // 3) Colored 5-char level like Full
        write_colored_level(&mut writer, meta.level())?;
        writer.write_char(' ')?;

        // 4) Span scope chain (bold names, fields in braces, dimmed ':')
        if let Some(scope) = ctx.event_scope() {
            let mut saw_any = false;
            for span in scope.from_root() {
                write_bold(&mut writer, span.metadata().name())?;
                saw_any = true;
                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.is_empty() {
                        write_bold(&mut writer, "{")?;
                        write!(writer, "{}", fields)?;
                        write_bold(&mut writer, "}")?;
                    }
                }
                if writer.has_ansi_escapes() {
                    write!(writer, "\x1b[2m:\x1b[0m")?;
                } else {
                    writer.write_char(':')?;
                }
            }
            if saw_any {
                writer.write_char(' ')?;
            }
        }

        // 5) Target (dimmed), then a space
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m{}\x1b[0m\x1b[2m:\x1b[0m ", meta.target())?;
        } else {
            write!(writer, "{}: ", meta.target())?;
        }

        // 6) Event fields
        ctx.format_fields(writer.by_ref(), event)?;

        // 7) Newline
        writeln!(writer)
    }
}

/// Write the verbosity level with the same coloring/alignment as the Full formatter.
fn write_colored_level(writer: &mut Writer<'_>, level: &Level) -> fmt::Result {
    if writer.has_ansi_escapes() {
        // Basic ANSI color sequences; reset with \x1b[0m
        let (color, text) = match *level {
            Level::TRACE => ("\x1b[35m", "TRACE"), // purple
            Level::DEBUG => ("\x1b[34m", "DEBUG"), // blue
            Level::INFO => ("\x1b[32m", " INFO"),  // green, note leading space
            Level::WARN => ("\x1b[33m", " WARN"),  // yellow, note leading space
            Level::ERROR => ("\x1b[31m", "ERROR"), // red
        };
        write!(writer, "{}{}\x1b[0m", color, text)
    } else {
        // Right-pad to width 5 like Full's non-ANSI mode
        match *level {
            Level::TRACE => write!(writer, "{:>5}", "TRACE"),
            Level::DEBUG => write!(writer, "{:>5}", "DEBUG"),
            Level::INFO => write!(writer, "{:>5}", " INFO"),
            Level::WARN => write!(writer, "{:>5}", " WARN"),
            Level::ERROR => write!(writer, "{:>5}", "ERROR"),
        }
    }
}

fn write_dimmed(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[2m{}\x1b[0m", s)
    } else {
        write!(writer, "{}", s)
    }
}

fn write_bold(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "\x1b[1m{}\x1b[0m", s)
    } else {
        write!(writer, "{}", s)
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
