#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

// A self-contained SDL2 demo showing how to keep a consistent aspect ratio
// with letterboxing/pillarboxing in a resizable window.
//
// This uses SDL2's logical size feature, which automatically sets a viewport
// to preserve the target aspect ratio and adds black bars as needed.
// We also clear the full window to black and then clear the logical viewport
// to a content color, so bars remain visibly black.

const LOGICAL_WIDTH: u32 = 320; // target content width
const LOGICAL_HEIGHT: u32 = 180; // target content height (16:9)

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    // Create a resizable window
    let window = video
        .window("SDL2 Aspect Ratio Demo", 960, 540)
        .resizable()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    // Set the desired logical (virtual) resolution. SDL will letterbox/pillarbox
    // as needed to preserve this aspect ratio when the window is resized.
    canvas
        .set_logical_size(LOGICAL_WIDTH, LOGICAL_HEIGHT)
        .map_err(|e| e.to_string())?;
    // Optional: uncomment to enforce integer scaling only (more retro look)
    // canvas.set_integer_scale(true)?;

    let mut events = sdl.event_pump()?;

    let mut running = true;
    let start = Instant::now();
    let mut last_log = Instant::now();

    while running {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                Event::Window { win_event, .. } => {
                    // Periodically log window size and the computed viewport
                    // to demonstrate how letterboxing/pillarboxing behaves.
                    use sdl2::event::WindowEvent;
                    match win_event {
                        WindowEvent::Resized(_, _)
                        | WindowEvent::SizeChanged(_, _)
                        | WindowEvent::Maximized
                        | WindowEvent::Restored => {
                            if last_log.elapsed() > Duration::from_millis(250) {
                                let out_size = canvas.output_size()?;
                                let viewport = canvas.viewport();
                                println!(
                                    "window={}x{}, viewport x={}, y={}, w={}, h={}",
                                    out_size.0,
                                    out_size.1,
                                    viewport.x(),
                                    viewport.y(),
                                    viewport.width(),
                                    viewport.height()
                                );
                                last_log = Instant::now();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // 1) Clear the entire window to black (no viewport) so the bars are black
        canvas.set_viewport(None);
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // 2) Re-apply logical size so SDL sets a viewport that preserves aspect
        //    ratio. Clearing now only affects the letterboxed content area.
        canvas
            .set_logical_size(LOGICAL_WIDTH, LOGICAL_HEIGHT)
            .map_err(|e| e.to_string())?;

        // Fill the content area with a background color to differentiate from bars
        canvas.set_draw_color(Color::RGB(30, 30, 40));
        canvas.clear();

        // Draw a simple grid to visualize scaling clearly
        canvas.set_draw_color(Color::RGB(60, 60, 90));
        let step = 20i32;
        for x in (0..=LOGICAL_WIDTH as i32).step_by(step as usize) {
            let _ = canvas.draw_line(sdl2::rect::Point::new(x, 0), sdl2::rect::Point::new(x, LOGICAL_HEIGHT as i32));
        }
        for y in (0..=LOGICAL_HEIGHT as i32).step_by(step as usize) {
            let _ = canvas.draw_line(sdl2::rect::Point::new(0, y), sdl2::rect::Point::new(LOGICAL_WIDTH as i32, y));
        }

        // Draw a border around the logical content area
        canvas.set_draw_color(Color::RGB(200, 200, 220));
        let border = Rect::new(0, 0, LOGICAL_WIDTH, LOGICAL_HEIGHT);
        canvas.draw_rect(border)?;

        // Draw a moving box to demonstrate dynamic content staying within aspect
        let elapsed_ms = start.elapsed().as_millis() as i32;
        let t = (elapsed_ms / 8) % LOGICAL_WIDTH as i32;
        let box_rect = Rect::new(t - 10, (LOGICAL_HEIGHT as i32 / 2) - 10, 20, 20);
        canvas.set_draw_color(Color::RGB(255, 140, 0));
        canvas.fill_rect(box_rect).ok();

        canvas.present();
    }

    Ok(())
}
