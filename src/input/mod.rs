use std::collections::HashMap;

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{Commands, NonSendMut, Res},
};
use sdl2::{event::Event, keyboard::Keycode, EventPump};

use crate::{entity::direction::Direction, game::events::GameEvent, input::commands::GameCommand};

pub mod commands;

#[derive(Debug, Clone, Resource)]
pub struct Bindings {
    key_bindings: HashMap<Keycode, GameCommand>,
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

        Self { key_bindings }
    }
}

pub fn handle_input(bindings: Res<Bindings>, mut writer: EventWriter<GameEvent>, mut pump: NonSendMut<&'static mut EventPump>) {
    for event in pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                writer.write(GameEvent::Command(GameCommand::Exit));
            }
            Event::KeyDown { keycode: Some(key), .. } => {
                let command = bindings.key_bindings.get(&key).copied();
                if let Some(command) = command {
                    tracing::info!("triggering command: {:?}", command);
                    writer.write(GameEvent::Command(command));
                }
            }
            _ => {}
        }
    }
}
