use pacman::systems::profiling::{SystemId, SystemTimings};
use speculoos::prelude::*;
use std::time::Duration;
use strum::IntoEnumIterator;

macro_rules! assert_close {
    ($actual:expr, $expected:expr, $concern:expr) => {
        let tolerance = Duration::from_micros(500);
        let diff = $actual.abs_diff($expected);
        assert_that(&(diff < tolerance)).is_true();
    };
}

#[test]
fn test_timing_statistics() {
    let timings = SystemTimings::default();

    // Add consecutive timing measurements (no skipped ticks to avoid zero padding)
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(10), 1);
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(12), 2);
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(8), 3);

    // Add consecutive timing measurements for another system
    timings.add_timing(SystemId::Blinking, Duration::from_millis(3), 1);
    timings.add_timing(SystemId::Blinking, Duration::from_millis(2), 2);
    timings.add_timing(SystemId::Blinking, Duration::from_millis(1), 3);

    {
        let stats = timings.get_stats(3);
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
    timings.add_timing(SystemId::PlayerControls, Duration::from_millis(5), 1);

    let stats = timings.get_stats(1);

    // Verify all SystemId variants are present in the stats
    let expected_count = SystemId::iter().count();
    assert_that(&stats.len()).is_equal_to(expected_count);

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
fn test_total_system_timing() {
    let timings = SystemTimings::default();

    // Add some timing data to the Total system
    timings.add_total_timing(Duration::from_millis(16), 1);
    timings.add_total_timing(Duration::from_millis(18), 2);
    timings.add_total_timing(Duration::from_millis(14), 3);

    let stats = timings.get_stats(3);
    let (avg, std_dev) = stats.get(&SystemId::Total).unwrap();

    // Should have 16ms average (16+18+14)/3 = 16ms
    assert_close!(*avg, Duration::from_millis(16), "Total system average timing");
    // Should have some standard deviation
    assert_that(&(*std_dev > Duration::ZERO)).is_true();
}
