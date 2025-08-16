use num_width::NumberWidth;
use smallvec::SmallVec;
use std::time::Duration;
use strum::EnumCount;

use crate::systems::profiling::SystemId;

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
