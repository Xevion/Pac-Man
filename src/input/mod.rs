use std::collections::HashMap;

use sdl2::{event::Event, keyboard::Keycode};

use crate::{entity::direction::Direction, input::commands::GameCommand};

pub mod commands;

#[derive(Debug, Clone, Default)]
pub struct InputSystem {
    key_bindings: HashMap<Keycode, GameCommand>,
}

impl InputSystem {
    pub fn new() -> Self {
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

    /// Handles an event and returns a command if one is bound to the event.
    pub fn handle_event(&self, event: &Event) -> Option<GameCommand> {
        match event {
            Event::Quit { .. } => Some(GameCommand::Exit),
            Event::KeyDown { keycode: Some(key), .. } => self.key_bindings.get(key).copied(),
            _ => None,
        }
    }
}
