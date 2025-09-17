//! Custom tracing formatter
use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt;
use time::macros::format_description;
use time::{format_description::FormatItem, OffsetDateTime};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;
use yansi::Paint;

// Cached format description for timestamps
const TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second].[subsecond digits:5]");

/// A custom formatter with enhanced timestamp formatting
///
/// Re-implementation of the Full formatter with improved timestamp display.
pub struct CustomPrettyFormatter;

impl<S, N> FormatEvent<S, N> for CustomPrettyFormatter
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

        // 2) Colored 5-char level like Full
        write_colored_level(&mut writer, meta.level())?;
        writer.write_char(' ')?;

        // 3) Span scope chain (bold names, fields in braces, dimmed ':')
        if let Some(scope) = ctx.event_scope() {
            let mut saw_any = false;
            for span in scope.from_root() {
                write_bold(&mut writer, span.metadata().name())?;
                saw_any = true;
                write_dimmed(&mut writer, ":")?;

                let ext = span.extensions();
                if let Some(fields) = &ext.get::<FormattedFields<N>>() {
                    if !fields.fields.is_empty() {
                        write_bold(&mut writer, "{")?;
                        writer.write_str(fields.fields.as_str())?;
                        write_bold(&mut writer, "}")?;
                    }
                }
                write_dimmed(&mut writer, ":")?;
            }
            if saw_any {
                writer.write_char(' ')?;
            }
        }

        // 4) Target (dimmed), then a space
        if writer.has_ansi_escapes() {
            write!(writer, "{}: ", Paint::new(meta.target()).dim())?;
        } else {
            write!(writer, "{}: ", meta.target())?;
        }

        // 5) Event fields
        ctx.format_fields(writer.by_ref(), event)?;

        // 6) Newline
        writeln!(writer)
    }
}

/// A custom JSON formatter that flattens fields to root level
///
/// Outputs logs in the format:
/// { "message": "...", "level": "...", "customAttribute": "..." }
pub struct CustomJsonFormatter;

impl<S, N> FormatEvent<S, N> for CustomJsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(&self, ctx: &FmtContext<'_, S, N>, mut writer: Writer<'_>, event: &Event<'_>) -> fmt::Result {
        let meta = event.metadata();

        #[derive(Serialize)]
        struct EventFields {
            message: String,
            level: String,
            target: String,
            #[serde(flatten)]
            spans: Map<String, Value>,
            #[serde(flatten)]
            fields: Map<String, Value>,
        }

        let (message, fields, spans) = {
            let mut message: Option<String> = None;
            let mut fields: Map<String, Value> = Map::new();
            let mut spans: Map<String, Value> = Map::new();

            struct FieldVisitor<'a> {
                message: &'a mut Option<String>,
                fields: &'a mut Map<String, Value>,
            }

            impl<'a> Visit for FieldVisitor<'a> {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(format!("{:?}", value));
                    } else {
                        self.fields.insert(key.to_string(), Value::String(format!("{:?}", value)));
                    }
                }

                fn record_str(&mut self, field: &Field, value: &str) {
                    let key = field.name();
                    if key == "message" {
                        *self.message = Some(value.to_string());
                    } else {
                        self.fields.insert(key.to_string(), Value::String(value.to_string()));
                    }
                }

                fn record_i64(&mut self, field: &Field, value: i64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields
                            .insert(key.to_string(), Value::Number(serde_json::Number::from(value)));
                    }
                }

                fn record_u64(&mut self, field: &Field, value: u64) {
                    let key = field.name();
                    if key != "message" {
                        self.fields
                            .insert(key.to_string(), Value::Number(serde_json::Number::from(value)));
                    }
                }

                fn record_bool(&mut self, field: &Field, value: bool) {
                    let key = field.name();
                    if key != "message" {
                        self.fields.insert(key.to_string(), Value::Bool(value));
                    }
                }
            }

            let mut visitor = FieldVisitor {
                message: &mut message,
                fields: &mut fields,
            };
            event.record(&mut visitor);

            // Collect span information from the span hierarchy
            if let Some(scope) = ctx.event_scope() {
                for span in scope.from_root() {
                    let span_name = span.metadata().name().to_string();
                    let mut span_fields: Map<String, Value> = Map::new();

                    // Try to extract fields from FormattedFields
                    let ext = span.extensions();
                    if let Some(formatted_fields) = ext.get::<FormattedFields<N>>() {
                        // Try to parse as JSON first
                        if let Ok(json_fields) = serde_json::from_str::<Map<String, Value>>(formatted_fields.fields.as_str()) {
                            span_fields.extend(json_fields);
                        } else {
                            // If not valid JSON, treat the entire field string as a single field
                            span_fields.insert("raw".to_string(), Value::String(formatted_fields.fields.as_str().to_string()));
                        }
                    }

                    // Insert span as a nested object directly into the spans map
                    spans.insert(span_name, Value::Object(span_fields));
                }
            }

            (message, fields, spans)
        };

        let json = EventFields {
            message: message.unwrap_or_default(),
            level: meta.level().to_string(),
            target: meta.target().to_string(),
            spans,
            fields,
        };

        writeln!(
            writer,
            "{}",
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
        )
    }
}

/// Write the verbosity level with the same coloring/alignment as the Full formatter.
fn write_colored_level(writer: &mut Writer<'_>, level: &Level) -> fmt::Result {
    if writer.has_ansi_escapes() {
        let paint = match *level {
            Level::TRACE => Paint::new("TRACE").magenta(),
            Level::DEBUG => Paint::new("DEBUG").blue(),
            Level::INFO => Paint::new(" INFO").green(),
            Level::WARN => Paint::new(" WARN").yellow(),
            Level::ERROR => Paint::new("ERROR").red(),
        };
        write!(writer, "{}", paint)
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
        write!(writer, "{}", Paint::new(s).dim())
    } else {
        write!(writer, "{}", s)
    }
}

fn write_bold(writer: &mut Writer<'_>, s: impl fmt::Display) -> fmt::Result {
    if writer.has_ansi_escapes() {
        write!(writer, "{}", Paint::new(s).bold())
    } else {
        write!(writer, "{}", s)
    }
}
