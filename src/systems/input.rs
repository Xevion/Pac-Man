use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{NonSendMut, Res, ResMut},
};
use glam::Vec2;
use sdl2::{event::Event, keyboard::Keycode, EventPump};
use smallvec::{smallvec, SmallVec};

use crate::systems::components::DeltaTime;
use crate::{
    events::{GameCommand, GameEvent},
    map::direction::Direction,
};

#[derive(Resource, Default, Debug, Copy, Clone)]
pub enum CursorPosition {
    #[default]
    None,
    Some {
        position: Vec2,
        remaining_time: f32,
    },
}

#[derive(Resource, Default, Debug)]
pub struct TouchState {
    pub active_touch: Option<TouchData>,
}

#[derive(Debug, Clone)]
pub struct TouchData {
    pub finger_id: i64,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
    pub current_direction: Option<Direction>,
}

impl TouchData {
    pub fn new(finger_id: i64, start_pos: Vec2) -> Self {
        Self {
            finger_id,
            start_pos,
            current_pos: start_pos,
            current_direction: None,
        }
    }

    pub fn update_position(&mut self, new_pos: Vec2) -> Option<Direction> {
        self.current_pos = new_pos;
        let delta = new_pos - self.start_pos;

        // Minimum threshold for direction detection (in pixels)
        const THRESHOLD: f32 = 20.0;

        if delta.length() < THRESHOLD {
            self.current_direction = None;
            return None;
        }

        // Determine primary direction based on larger component
        let direction = if delta.x.abs() > delta.y.abs() {
            if delta.x > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if delta.y > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        };

        self.current_direction = Some(direction);
        Some(direction)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Bindings {
    key_bindings: HashMap<Keycode, GameCommand>,
    movement_keys: HashSet<Keycode>,
    pressed_movement_keys: Vec<Keycode>,
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
            pressed_movement_keys: Vec::new(),
        }
    }
}

/// A simplified input event used for deterministic testing and logic reuse
/// without depending on SDL's event pump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleKeyEvent {
    KeyDown(Keycode),
    KeyUp(Keycode),
}

/// Processes a frame's worth of simplified key events and returns the resulting
/// `GameEvent`s that would be emitted by the input system for that frame.
///
/// This mirrors the behavior of `input_system` for keyboard-related logic:
/// - KeyDown emits the bound command immediately (movement or otherwise)
/// - Tracks pressed movement keys in order to continue movement on subsequent frames
/// - KeyUp removes movement keys; if another movement key remains, it resumes
pub fn process_simple_key_events(bindings: &mut Bindings, frame_events: &[SimpleKeyEvent]) -> Vec<GameEvent> {
    let mut emitted_events = Vec::new();
    let mut movement_key_pressed = false;

    for event in frame_events {
        match *event {
            SimpleKeyEvent::KeyDown(key) => {
                if let Some(command) = bindings.key_bindings.get(&key).copied() {
                    emitted_events.push(GameEvent::Command(command));
                }

                if bindings.movement_keys.contains(&key) {
                    movement_key_pressed = true;
                    if !bindings.pressed_movement_keys.contains(&key) {
                        bindings.pressed_movement_keys.push(key);
                    }
                }
            }
            SimpleKeyEvent::KeyUp(key) => {
                if bindings.movement_keys.contains(&key) {
                    bindings.pressed_movement_keys.retain(|&k| k != key);
                }
            }
        }
    }

    if !movement_key_pressed {
        if let Some(&last_movement_key) = bindings.pressed_movement_keys.last() {
            if let Some(command) = bindings.key_bindings.get(&last_movement_key).copied() {
                emitted_events.push(GameEvent::Command(command));
            }
        }
    }

    emitted_events
}

pub fn input_system(
    delta_time: Res<DeltaTime>,
    mut bindings: ResMut<Bindings>,
    mut writer: EventWriter<GameEvent>,
    mut pump: NonSendMut<EventPump>,
    mut cursor: ResMut<CursorPosition>,
    mut touch_state: ResMut<TouchState>,
) {
    let mut cursor_seen = false;
    // Collect all events for this frame.
    let frame_events: SmallVec<[Event; 3]> = pump.poll_iter().collect();

    // Warn if the smallvec was heap allocated due to exceeding stack capacity
    #[cfg(debug_assertions)]
    if frame_events.len() > frame_events.capacity() {
        tracing::warn!(
            "More than {} events in a frame, consider adjusting stack capacity: {:?}",
            frame_events.capacity(),
            frame_events
        );
    }

    // Handle non-keyboard events inline and build a simplified keyboard event stream.
    let mut simple_key_events: SmallVec<[SimpleKeyEvent; 3]> = smallvec![];
    for event in &frame_events {
        match *event {
            Event::Quit { .. } => {
                writer.write(GameEvent::Command(GameCommand::Exit));
            }
            Event::MouseMotion { x, y, .. } => {
                *cursor = CursorPosition::Some {
                    position: Vec2::new(x as f32, y as f32),
                    remaining_time: 0.20,
                };
                cursor_seen = true;

                // Handle mouse motion as touch motion for desktop testing
                if let Some(ref mut touch_data) = touch_state.active_touch {
                    if let Some(direction) = touch_data.update_position(Vec2::new(x as f32, y as f32)) {
                        writer.write(GameEvent::Command(GameCommand::MovePlayer(direction)));
                    }
                }
            }
            // Handle mouse events as touch for desktop testing
            Event::MouseButtonDown { x, y, .. } => {
                let pos = Vec2::new(x as f32, y as f32);
                touch_state.active_touch = Some(TouchData::new(0, pos)); // Use ID 0 for mouse
            }
            Event::MouseButtonUp { .. } => {
                touch_state.active_touch = None;
            }
            // Handle actual touch events for mobile
            Event::FingerDown { finger_id, x, y, .. } => {
                // Convert normalized coordinates (0.0-1.0) to screen coordinates
                let screen_x = x * crate::constants::CANVAS_SIZE.x as f32;
                let screen_y = y * crate::constants::CANVAS_SIZE.y as f32;
                let pos = Vec2::new(screen_x, screen_y);
                touch_state.active_touch = Some(TouchData::new(finger_id, pos));
            }
            Event::FingerMotion { finger_id, x, y, .. } => {
                if let Some(ref mut touch_data) = touch_state.active_touch {
                    if touch_data.finger_id == finger_id {
                        let screen_x = x * crate::constants::CANVAS_SIZE.x as f32;
                        let screen_y = y * crate::constants::CANVAS_SIZE.y as f32;
                        if let Some(direction) = touch_data.update_position(Vec2::new(screen_x, screen_y)) {
                            writer.write(GameEvent::Command(GameCommand::MovePlayer(direction)));
                        }
                    }
                }
            }
            Event::FingerUp { finger_id, .. } => {
                if let Some(ref touch_data) = touch_state.active_touch {
                    if touch_data.finger_id == finger_id {
                        touch_state.active_touch = None;
                    }
                }
            }
            Event::KeyDown { keycode, repeat, .. } => {
                if let Some(key) = keycode {
                    if repeat {
                        continue;
                    }
                    simple_key_events.push(SimpleKeyEvent::KeyDown(key));
                }
            }
            Event::KeyUp { keycode, repeat, .. } => {
                if let Some(key) = keycode {
                    if repeat {
                        continue;
                    }
                    simple_key_events.push(SimpleKeyEvent::KeyUp(key));
                }
            }
            _ => {
                tracing::warn!("Unhandled event, consider disabling: {:?}", event);
            }
        }
    }

    // Delegate keyboard handling to shared logic used by tests and production.
    let emitted = process_simple_key_events(&mut bindings, &simple_key_events);
    for event in emitted {
        writer.write(event);
    }

    if let (false, CursorPosition::Some { remaining_time, .. }) = (cursor_seen, &mut *cursor) {
        *remaining_time -= delta_time.0;
        if *remaining_time <= 0.0 {
            *cursor = CursorPosition::None;
        }
    }
}
