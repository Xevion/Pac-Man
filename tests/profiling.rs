use pacman::systems::profiling::{SystemId, SystemTimings};
use std::time::Duration;
use strum::IntoEnumIterator;

macro_rules! assert_close {
    ($actual:expr, $expected:expr, $concern:expr) => {
        let tolerance = Duration::from_micros(500);
        let diff = $actual.abs_diff($expected);
        assert!(
            diff < tolerance,
            "Expected {expected:?} ± {tolerance:.0?}, got {actual:?}, off by {diff:?} ({concern})",
            concern = $concern,
            expected = $expected,
            actual = $actual,
            tolerance = tolerance,
            diff = diff
        );
    };
}

#[test]
fn test_timing_statistics() {
    let timings = SystemTimings::default();

    // 10ms average, 2ms std dev
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(10));
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(12));
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(8));

    // 2ms average, 1ms std dev
    timings.add_timing(SystemId::Blinking, Duration::from_millis(3));
    timings.add_timing(SystemId::Blinking, Duration::from_millis(2));
    timings.add_timing(SystemId::Blinking, Duration::from_millis(1));

    {
        let stats = timings.get_stats();
        let (avg, std_dev) = stats.get(&SystemId::PlayerControls).unwrap();

        assert_close!(*avg, Duration::from_millis(10), "PlayerControls average timing");
        assert_close!(*std_dev, Duration::from_millis(2), "PlayerControls standard deviation timing");
    }

    // Note: get_total_stats() was removed as we now use the Total system directly
    // This test now focuses on individual system statistics
}

#[test]
fn test_default_zero_timing_for_unused_systems() {
    let timings = SystemTimings::default();

    // Add timing data for only one system
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(5));

    let stats = timings.get_stats();

    // Verify all SystemId variants are present in the stats
    let expected_count = SystemId::iter().count();
    assert_eq!(stats.len(), expected_count, "All SystemId variants should be in stats");

    // Verify that the system with data has non-zero timing
    let (avg, std_dev) = stats.get(&SystemId::PlayerControls).unwrap();
    assert_close!(*avg, Duration::from_millis(5), "System with data should have correct timing");
    assert_close!(*std_dev, Duration::ZERO, "Single measurement should have zero std dev");

    // Verify that all other systems have zero timing (excluding Total which is special)
    for id in SystemId::iter() {
        if id != SystemId::PlayerControls && id != SystemId::Total {
            let (avg, std_dev) = stats.get(&id).unwrap();
            assert_close!(
                *avg,
                Duration::ZERO,
                format!("Unused system {:?} should have zero avg timing", id)
            );
            assert_close!(
                *std_dev,
                Duration::ZERO,
                format!("Unused system {:?} should have zero std dev", id)
            );
        }
    }
}

#[test]
fn test_pre_populated_timing_entries() {
    let timings = SystemTimings::default();

    // Verify that we can add timing to any SystemId without panicking
    // (this would fail with the old implementation if the entry didn't exist)
    for id in SystemId::iter() {
        timings.add_timing(id, Duration::from_nanos(1));
    }

    // Verify all systems now have non-zero timing
    let stats = timings.get_stats();
    for id in SystemId::iter() {
        let (avg, _) = stats.get(&id).unwrap();
        assert!(
            *avg > Duration::ZERO,
            "System {:?} should have non-zero timing after add_timing",
            id
        );
    }
}

#[test]
fn test_total_system_timing() {
    let timings = SystemTimings::default();

    // Add some timing data to the Total system
    timings.add_total_timing(Duration::from_millis(16));
    timings.add_total_timing(Duration::from_millis(18));
    timings.add_total_timing(Duration::from_millis(14));

    let stats = timings.get_stats();
    let (avg, std_dev) = stats.get(&SystemId::Total).unwrap();

    // Should have 16ms average (16+18+14)/3 = 16ms
    assert_close!(*avg, Duration::from_millis(16), "Total system average timing");
    // Should have some standard deviation
    assert!(
        *std_dev > Duration::ZERO,
        "Total system should have non-zero std dev with multiple measurements"
    );
}
