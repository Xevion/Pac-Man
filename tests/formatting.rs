use pacman::systems::formatting::format_timing_display;
use std::time::Duration;

#[test]
fn test_basic_formatting() {
    let timing_data = vec![
        ("60 FPS".to_string(), Duration::from_micros(1234), Duration::from_micros(567)),
        ("input".to_string(), Duration::from_micros(123), Duration::from_micros(45)),
        ("player".to_string(), Duration::from_micros(456), Duration::from_micros(123)),
        ("movement".to_string(), Duration::from_micros(789), Duration::from_micros(234)),
        ("render".to_string(), Duration::from_micros(12), Duration::from_micros(3)),
        ("debug".to_string(), Duration::from_nanos(1000000), Duration::from_nanos(1000)),
    ];

    let result = format_timing_display(timing_data);
    println!("Basic formatting test:");
    println!("{}", result);
    println!();
}

#[test]
fn test_desired_format() {
    // This test represents the exact format you want to achieve
    let timing_data = vec![
        ("total".to_string(), Duration::from_micros(1230), Duration::from_micros(570)),
        ("input".to_string(), Duration::from_micros(120), Duration::from_micros(50)),
        ("player".to_string(), Duration::from_micros(460), Duration::from_micros(120)),
        ("movement".to_string(), Duration::from_micros(790), Duration::from_micros(230)),
        ("render".to_string(), Duration::from_micros(10), Duration::from_micros(3)),
        ("debug".to_string(), Duration::from_nanos(1000000), Duration::from_nanos(1000)),
    ];

    let result = format_timing_display(timing_data);
    println!("Desired format test:");
    println!("{}", result);
    println!();

    // Expected output should look like:
    // total    :    1.23 ms ±    0.57 ms
    // input    :    0.12 ms ±    0.05 ms
    // player   :    0.46 ms ±    0.12 ms
    // movement :    0.79 ms ±    0.23 ms
    // render   :    0.01 ms ±    0.003ms
    // debug    :    0.001ms ±    0.000ms
}

#[test]
fn test_mixed_units() {
    let timing_data = vec![
        ("60 FPS".to_string(), Duration::from_millis(16), Duration::from_micros(500)),
        (
            "fast_system".to_string(),
            Duration::from_nanos(500000),
            Duration::from_nanos(100000),
        ),
        (
            "medium_system".to_string(),
            Duration::from_micros(2500),
            Duration::from_micros(500),
        ),
        ("slow_system".to_string(), Duration::from_millis(5), Duration::from_millis(1)),
    ];

    let result = format_timing_display(timing_data);
    println!("Mixed units test:");
    println!("{}", result);
    println!();
}

#[test]
fn test_trailing_zeros() {
    let timing_data = vec![
        ("60 FPS".to_string(), Duration::from_micros(1000), Duration::from_micros(500)),
        ("exact_ms".to_string(), Duration::from_millis(1), Duration::from_micros(100)),
        ("exact_us".to_string(), Duration::from_micros(1), Duration::from_nanos(100000)),
        ("exact_ns".to_string(), Duration::from_nanos(1000), Duration::from_nanos(100)),
    ];

    let result = format_timing_display(timing_data);
    println!("Trailing zeros test:");
    println!("{}", result);
    println!();
}

#[test]
fn test_edge_cases() {
    let timing_data = vec![
        ("60 FPS".to_string(), Duration::from_nanos(1), Duration::from_nanos(1)),
        ("very_small".to_string(), Duration::from_nanos(100), Duration::from_nanos(50)),
        ("very_large".to_string(), Duration::from_secs(1), Duration::from_millis(100)),
        ("zero_time".to_string(), Duration::ZERO, Duration::ZERO),
    ];

    let result = format_timing_display(timing_data);
    println!("Edge cases test:");
    println!("{}", result);
    println!();
}

#[test]
fn test_variable_name_lengths() {
    let timing_data = vec![
        ("60 FPS".to_string(), Duration::from_micros(1234), Duration::from_micros(567)),
        ("a".to_string(), Duration::from_micros(123), Duration::from_micros(45)),
        (
            "very_long_system_name".to_string(),
            Duration::from_micros(456),
            Duration::from_micros(123),
        ),
        ("medium".to_string(), Duration::from_micros(789), Duration::from_micros(234)),
    ];

    let result = format_timing_display(timing_data);
    println!("Variable name lengths test:");
    println!("{}", result);
    println!();
}

#[test]
fn test_empty_input() {
    let timing_data = vec![];
    let result = format_timing_display(timing_data);
    assert_eq!(result, "");
    println!("Empty input test: PASS");
}

#[test]
fn test_single_entry() {
    let timing_data = vec![("60 FPS".to_string(), Duration::from_micros(1234), Duration::from_micros(567))];

    let result = format_timing_display(timing_data);
    println!("Single entry test:");
    println!("{}", result);
    println!();
}
