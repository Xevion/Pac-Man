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

    {
        let (total_avg, total_std) = timings.get_total_stats();
        assert_close!(total_avg, Duration::from_millis(2), "Total average timing across all systems");
        assert_close!(
            total_std,
            Duration::from_millis(7),
            "Total standard deviation timing across all systems"
        );
    }
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

    // Verify that all other systems have zero timing
    for id in SystemId::iter() {
        if id != SystemId::PlayerControls {
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
