use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::game::Game;
use crate::textures::TextureManager;
use sdl2::event::{Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use std::time::{Duration, Instant};

#[cfg(target_os = "emscripten")]
pub mod emscripten;

mod constants;
mod direction;
mod game;
mod pacman;
mod textures;
mod entity;
mod animation;

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
        .build()
        .expect("Could not initialize window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Could not build canvas");

    canvas
        .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Could not set logical size");

    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&mut canvas, &texture_creator);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not get SDL EventPump");

    game.draw();
    game.tick();

    let loop_time = Duration::from_millis(1000 / 60);

    let mut main_loop = || {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                // Handle quitting keys or window close
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => return false,
                Event::KeyDown { keycode , .. } => {
                    game.keyboard_event(keycode.unwrap());
                },
                _ => {}
            }
        }

        let tick_time = {
            let start = Instant::now();
            game.tick();
            start.elapsed()
        };

        let draw_time = {
            let start = Instant::now();
            game.draw();
            start.elapsed()
        };

        if start.elapsed() < loop_time {
            ::std::thread::sleep(loop_time - start.elapsed());
        } else {
            println!("Game loop behind schedule by: {:?}", start.elapsed() - loop_time);
        }

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
