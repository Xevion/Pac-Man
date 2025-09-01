use pacman::systems::profiling::{SystemId, SystemTimings};
use std::time::Duration;

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
    fn close_enough(a: Duration, b: Duration) -> bool {
        if a > b {
            a - b < Duration::from_micros(500) // 0.1ms
        } else {
            b - a < Duration::from_micros(500)
        }
    }

    let stats = timings.get_stats();
    let (avg, std_dev) = stats.get(&SystemId::PlayerControls).unwrap();

    // Average should be 10ms, standard deviation should be small
    assert!(close_enough(*avg, Duration::from_millis(10)), "avg: {:?}", avg);
    assert!(close_enough(*std_dev, Duration::from_millis(2)), "std_dev: {:?}", std_dev);

    let (total_avg, total_std) = timings.get_total_stats();
    assert!(
        close_enough(total_avg, Duration::from_millis(18)),
        "total_avg: {:?}",
        total_avg
    );
    assert!(
        close_enough(total_std, Duration::from_millis(12)),
        "total_std: {:?}",
        total_std
    );
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
