#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use circular_buffer::CircularBuffer;
use pacman::constants::CANVAS_SIZE;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("SDL2 Timing Demo", CANVAS_SIZE.x, CANVAS_SIZE.y)
        .opengl()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(CANVAS_SIZE.x, CANVAS_SIZE.y)
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    // Store frame timings in milliseconds
    let mut frame_timings = CircularBuffer::<20_000, f64>::new();
    let mut last_report_time = Instant::now();
    let report_interval = Duration::from_millis(500);

    'running: loop {
        let frame_start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        // Clear the screen
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        // Record timing
        let frame_duration = frame_start_time.elapsed();
        frame_timings.push_back(frame_duration.as_secs_f64());

        // Report stats every `report_interval`
        let elapsed = last_report_time.elapsed();
        if elapsed >= report_interval {
            if !frame_timings.is_empty() {
                let count = frame_timings.len() as f64;
                let sum: f64 = frame_timings.iter().sum();
                let mean = sum / count;

                let variance = frame_timings
                    .iter()
                    .map(|value| {
                        let diff = mean - value;
                        diff * diff
                    })
                    .sum::<f64>()
                    / count;
                let std_dev = variance.sqrt();

                println!(
                    "Rendered {count} frames at {fps:.1} fps (last {elapsed:.2?}): mean={mean:.3?}, std_dev={std_dev:.3?}",
                    count = frame_timings.len(),
                    fps = count / elapsed.as_secs_f64(),
                    elapsed = elapsed,
                    mean = Duration::from_secs_f64(mean),
                    std_dev = Duration::from_secs_f64(std_dev),
                );
            }

            // Reset for next interval
            frame_timings.clear();
            last_report_time = Instant::now();
        }
    }

    Ok(())
}
