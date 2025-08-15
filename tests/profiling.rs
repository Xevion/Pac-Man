use pacman::systems::profiling::SystemTimings;
use std::time::Duration;

#[test]
fn test_timing_statistics() {
    let timings = SystemTimings::default();

    // Add some test data
    timings.add_timing("test_system", Duration::from_millis(10));
    timings.add_timing("test_system", Duration::from_millis(12));
    timings.add_timing("test_system", Duration::from_millis(8));

    let stats = timings.get_stats();
    let (avg, std_dev) = stats.get("test_system").unwrap();

    // Average should be 10ms, standard deviation should be small
    assert!((avg.as_millis() as f64 - 10.0).abs() < 1.0);
    assert!(std_dev.as_millis() > 0);

    let (total_avg, total_std) = timings.get_total_stats();
    assert!((total_avg.as_millis() as f64 - 10.0).abs() < 1.0);
    assert!(total_std.as_millis() > 0);
}

// #[test]
// fn test_window_size_limit() {
//     let timings = SystemTimings::default();

//     // Add more than 90 timings to test window size limit
//     for i in 0..100 {
//         timings.add_timing("test_system", Duration::from_millis(i));
//     }

//     let stats = timings.get_stats();
//     let (avg, _) = stats.get("test_system").unwrap();

//     // Should only keep the last 90 values, so average should be around 55ms
//     // (average of 10-99)
//     assert!((avg.as_millis() as f64 - 55.0).abs() < 5.0);
// }
