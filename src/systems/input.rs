use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{NonSendMut, ResMut},
};
use glam::Vec2;
use sdl2::{event::Event, keyboard::Keycode, EventPump};

use crate::systems::debug::CursorPosition;
use crate::{
    entity::direction::Direction,
    events::{GameCommand, GameEvent},
};

#[derive(Debug, Clone, Resource)]
pub struct Bindings {
    key_bindings: HashMap<Keycode, GameCommand>,
    movement_keys: HashSet<Keycode>,
    last_movement_key: Option<Keycode>,
}

impl Default for Bindings {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();

        // Player movement
        key_bindings.insert(Keycode::Up, GameCommand::MovePlayer(Direction::Up));
        key_bindings.insert(Keycode::W, GameCommand::MovePlayer(Direction::Up));
        key_bindings.insert(Keycode::Down, GameCommand::MovePlayer(Direction::Down));
        key_bindings.insert(Keycode::S, GameCommand::MovePlayer(Direction::Down));
        key_bindings.insert(Keycode::Left, GameCommand::MovePlayer(Direction::Left));
        key_bindings.insert(Keycode::A, GameCommand::MovePlayer(Direction::Left));
        key_bindings.insert(Keycode::Right, GameCommand::MovePlayer(Direction::Right));
        key_bindings.insert(Keycode::D, GameCommand::MovePlayer(Direction::Right));

        // Game actions
        key_bindings.insert(Keycode::P, GameCommand::TogglePause);
        key_bindings.insert(Keycode::Space, GameCommand::ToggleDebug);
        key_bindings.insert(Keycode::M, GameCommand::MuteAudio);
        key_bindings.insert(Keycode::R, GameCommand::ResetLevel);
        key_bindings.insert(Keycode::Escape, GameCommand::Exit);
        key_bindings.insert(Keycode::Q, GameCommand::Exit);

        let movement_keys = HashSet::from([
            Keycode::W,
            Keycode::A,
            Keycode::S,
            Keycode::D,
            Keycode::Up,
            Keycode::Down,
            Keycode::Left,
            Keycode::Right,
        ]);

        Self {
            key_bindings,
            movement_keys,
            last_movement_key: None,
        }
    }
}

pub fn input_system(
    mut bindings: ResMut<Bindings>,
    mut writer: EventWriter<GameEvent>,
    mut pump: NonSendMut<&'static mut EventPump>,
    mut cursor: ResMut<CursorPosition>,
) {
    let mut movement_key_pressed = false;

    for event in pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                writer.write(GameEvent::Command(GameCommand::Exit));
            }
            Event::MouseMotion { x, y, .. } => {
                cursor.0 = Vec2::new(x as f32, y as f32);
            }
            Event::KeyUp {
                repeat: false,
                keycode: Some(key),
                ..
            } => {
                // If the last movement key was released, then forget it.
                if let Some(last_movement_key) = bindings.last_movement_key {
                    if last_movement_key == key {
                        bindings.last_movement_key = None;
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(key),
                repeat: false,
                ..
            } => {
                let command = bindings.key_bindings.get(&key).copied();
                if let Some(command) = command {
                    writer.write(GameEvent::Command(command));
                }

                if bindings.movement_keys.contains(&key) {
                    movement_key_pressed = true;
                    bindings.last_movement_key = Some(key);
                }
            }
            _ => {}
        }
    }

    if let Some(last_movement_key) = bindings.last_movement_key {
        if !movement_key_pressed {
            let command = bindings.key_bindings.get(&last_movement_key).copied();
            if let Some(command) = command {
                writer.write(GameEvent::Command(command));
            }
        }
    }
}
