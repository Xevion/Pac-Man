use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::{Keycode, Mod};
use std::time::Duration;
use crate::constants::{WINDOW_WIDTH, WINDOW_HEIGHT};

mod constants;
mod board;
mod game;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Pac-Man", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Could not initialize window");

    let mut canvas = window.into_canvas().build().expect("Could not build canvas");
    let texture_creator= canvas.texture_creator();

    let map_texture = texture_creator.load_texture("assets/map.png").expect("Could not load pacman texture");
    canvas.copy(&map_texture, None, None).expect("Could not render texture on canvas");

    let mut event_pump = sdl_context.event_pump().expect("Could not get SDL EventPump");
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    break 'main;
                }
                event @ Event::KeyDown {  .. } => {
                    println!("{:?}", event);
                },
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_millis(10));
    }
}