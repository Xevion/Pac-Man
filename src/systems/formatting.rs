use num_width::NumberWidth;
use std::time::Duration;

/// Formats timing data into a vector of strings with proper alignment
pub fn format_timing_display(timing_data: Vec<(String, Duration, Duration)>) -> String {
    if timing_data.is_empty() {
        return String::new();
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

    struct Entry {
        name: String,
        avg_int: u64,
        avg_decimal: u32,
        avg_unit: &'static str,
        std_int: u64,
        std_decimal: u32,
        std_unit: &'static str,
    }

    let entries = timing_data
        .iter()
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
        .collect::<Vec<_>>();

    let max_name_width = entries.iter().map(|e| e.name.len() as usize).max().unwrap_or(0);
    let max_avg_int_width = entries.iter().map(|e| e.avg_int.width() as usize).max().unwrap_or(0);
    let max_avg_decimal_width = entries
        .iter()
        .map(|e| e.avg_decimal.width() as usize)
        .max()
        .unwrap_or(0)
        .max(3);
    let max_std_int_width = entries.iter().map(|e| e.std_int.width() as usize).max().unwrap_or(0);
    let max_std_decimal_width = entries
        .iter()
        .map(|e| e.std_decimal.width() as usize)
        .max()
        .unwrap_or(0)
        .max(3);

    let mut output_lines = Vec::new();

    // Format each line using the calculated max widths for alignment
    for Entry {
        name,
        avg_int,
        avg_decimal,
        avg_unit,
        std_int,
        std_decimal,
        std_unit,
    } in entries.iter()
    {
        // Add exactly 4 spaces of padding before each number
        let avg_padding = " ".repeat(4);
        let std_padding = " ".repeat(4);

        output_lines.push(format!(
            "{name:max_name_width$} : {avg_int:max_avg_int_width$}.{avg_decimal:<max_avg_decimal_width$}{avg_unit} ± {std_int:max_std_int_width$}.{std_decimal:<max_std_decimal_width$}{std_unit}"
        ));
    }

    output_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::time::Duration;

    #[test]
    fn test_format_timing_display() {
        let timing_data = vec![
            ("total".to_string(), Duration::from_micros(1234), Duration::from_micros(570)),
            ("input".to_string(), Duration::from_micros(120), Duration::from_micros(45)),
            ("player".to_string(), Duration::from_micros(456), Duration::from_micros(123)),
            ("movement".to_string(), Duration::from_micros(789), Duration::from_micros(234)),
            ("render".to_string(), Duration::from_micros(12), Duration::from_micros(3)),
            ("debug".to_string(), Duration::from_nanos(460), Duration::from_nanos(557)),
        ];

        let result = format_timing_display(timing_data);
        let lines: Vec<&str> = result.lines().collect();

        // Verify we have the expected number of lines
        assert_eq!(lines.len(), 6);

        let expected = r#"
total    :   1.234ms ± 570.0  µs
input    : 120.0  µs ±  45.0  µs
player   : 456.0  µs ± 123.0  µs
movement : 789.0  µs ± 234.0  µs
render   :  12.0  µs ±   3.0  µs
debug    : 460.0  ns ± 557.0  ns
"#
        .trim();

        for (line, expected_line) in lines.iter().zip(expected.lines()) {
            assert_eq!(*line, expected_line);
        }

        // Print the result for manual inspection
        println!("Formatted output:");
        println!("{}", result);
    }
}
