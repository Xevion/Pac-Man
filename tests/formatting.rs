use pacman::systems::profiling::format_timing_display;
use speculoos::prelude::*;
use std::time::Duration;

fn get_timing_data() -> Vec<(String, Duration, Duration)> {
    vec![
        ("total".to_string(), Duration::from_micros(1234), Duration::from_micros(570)),
        ("input".to_string(), Duration::from_micros(120), Duration::from_micros(45)),
        ("player".to_string(), Duration::from_micros(456), Duration::from_micros(123)),
        ("movement".to_string(), Duration::from_micros(789), Duration::from_micros(234)),
        ("render".to_string(), Duration::from_micros(12), Duration::from_micros(3)),
        ("debug".to_string(), Duration::from_nanos(460), Duration::from_nanos(557)),
    ]
}

fn get_formatted_output() -> impl IntoIterator<Item = String> {
    format_timing_display(get_timing_data())
}

#[test]
fn test_complex_formatting_alignment() {
    let mut colon_positions = vec![];
    let mut first_decimal_positions = vec![];
    let mut second_decimal_positions = vec![];
    let mut first_unit_positions = vec![];
    let mut second_unit_positions = vec![];

    get_formatted_output().into_iter().for_each(|line| {
        let (mut got_decimal, mut got_unit) = (false, false);
        for (i, char) in line.chars().enumerate() {
            match char {
                ':' => colon_positions.push(i),
                '.' => {
                    if got_decimal {
                        second_decimal_positions.push(i);
                    } else {
                        first_decimal_positions.push(i);
                    }
                    got_decimal = true;
                }
                's' => {
                    if got_unit {
                        first_unit_positions.push(i);
                    } else {
                        second_unit_positions.push(i);
                        got_unit = true;
                    }
                }
                _ => {}
            }
        }
    });

    // Assert that all positions were found
    assert_that(
        &[
            &colon_positions,
            &first_decimal_positions,
            &second_decimal_positions,
            &first_unit_positions,
            &second_unit_positions,
        ]
        .iter()
        .all(|p| p.len() == 6),
    )
    .is_true();

    // Assert that all positions are the same
    assert_that(&colon_positions.iter().all(|&p| p == colon_positions[0])).is_true();
    assert_that(&first_decimal_positions.iter().all(|&p| p == first_decimal_positions[0])).is_true();
    assert_that(&second_decimal_positions.iter().all(|&p| p == second_decimal_positions[0])).is_true();
    assert_that(&first_unit_positions.iter().all(|&p| p == first_unit_positions[0])).is_true();
    assert_that(&second_unit_positions.iter().all(|&p| p == second_unit_positions[0])).is_true();
}

#[test]
fn test_format_timing_display_basic() {
    let timing_data = vec![
        ("render".to_string(), Duration::from_micros(1500), Duration::from_micros(200)),
        ("input".to_string(), Duration::from_micros(300), Duration::from_micros(50)),
        ("physics".to_string(), Duration::from_nanos(750), Duration::from_nanos(100)),
    ];

    let formatted = format_timing_display(timing_data);

    // Should have 3 lines (one for each system)
    assert_that(&formatted.len()).is_equal_to(3);

    // Each line should contain the system name
    assert_that(&formatted.iter().any(|line| line.contains("render"))).is_true();
    assert_that(&formatted.iter().any(|line| line.contains("input"))).is_true();
    assert_that(&formatted.iter().any(|line| line.contains("physics"))).is_true();

    // Each line should contain timing information with proper units
    for line in formatted.iter() {
        assert_that(&line.contains(":")).is_true();
        assert_that(&line.contains("±")).is_true();
    }
}

#[test]
fn test_format_timing_display_units() {
    let timing_data = vec![
        ("seconds".to_string(), Duration::from_secs(2), Duration::from_millis(100)),
        ("millis".to_string(), Duration::from_millis(15), Duration::from_micros(200)),
        ("micros".to_string(), Duration::from_micros(500), Duration::from_nanos(50)),
        ("nanos".to_string(), Duration::from_nanos(250), Duration::from_nanos(25)),
    ];

    let formatted = format_timing_display(timing_data);

    // Check that appropriate units are used
    let all_lines = formatted.join(" ");
    assert_that(&all_lines.contains("s")).is_true();
    assert_that(&all_lines.contains("ms")).is_true();
    assert_that(&all_lines.contains("µs")).is_true();
    assert_that(&all_lines.contains("ns")).is_true();
}

#[test]
fn test_format_timing_display_alignment() {
    let timing_data = vec![
        ("short".to_string(), Duration::from_micros(100), Duration::from_micros(10)),
        (
            "very_long_name".to_string(),
            Duration::from_micros(200),
            Duration::from_micros(20),
        ),
    ];

    let formatted = format_timing_display(timing_data);

    // Find colon positions - they should be aligned
    let colon_positions: Vec<usize> = formatted.iter().map(|line| line.find(':').unwrap_or(0)).collect();

    // All colons should be at the same position (aligned)
    if colon_positions.len() > 1 {
        let first_pos = colon_positions[0];
        assert_that(&colon_positions.iter().all(|&pos| pos == first_pos)).is_true();
    }
}
