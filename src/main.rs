use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use sdl2::event::{Event, WindowEvent};
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Texture, Canvas};
use std::time::Duration;

#[cfg(target_os = "emscripten")]
pub mod emscripten;

mod board;
mod constants;
mod game;

fn redraw(canvas: &mut Canvas<sdl2::video::Window>, tex: &Texture, i: u8) {
    canvas.set_draw_color(Color::RGB(i, i, i));
    canvas.clear();
    canvas
        .copy(tex, None, None)
        .expect("Could not render texture on canvas");
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Pac-Man", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .expect("Could not initialize window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Could not build canvas");
    let texture_creator = canvas.texture_creator();

    let map_texture = texture_creator
        .load_texture("assets/map.png")
        .expect("Could not load pacman texture");

    canvas
        .copy(&map_texture, None, None)
        .expect("Could not render texture on canvas");
    
    let mut i = 0u8;

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not get SDL EventPump");

    let mut main_loop = || {
        for event in event_pump.poll_iter() {
            match event {
                // Handle quitting keys or window close
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => return false,
                event @ Event::KeyDown { .. } => {
                    println!("{:?}", event);
                },
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        i = i.wrapping_add(1);

                        canvas.set_logical_size(width as u32, height as u32).unwrap();
                        redraw(&mut canvas, &map_texture, i);
                    }
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_millis(10));
        true
    };

    #[cfg(target_os = "emscripten")]
    use emscripten::emscripten;

    #[cfg(target_os = "emscripten")]
    emscripten::set_main_loop_callback(main_loop);

    #[cfg(not(target_os = "emscripten"))]
    loop {
        if !main_loop() {
            break;
        }
    }
}
