use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{NonSendMut, Res, ResMut},
};
use glam::Vec2;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    EventPump,
};
use smallvec::{smallvec, SmallVec};

use crate::systems::components::DeltaTime;
use crate::{
    events::{GameCommand, GameEvent},
    map::direction::Direction,
};

// Touch input constants
pub const TOUCH_DIRECTION_THRESHOLD: f32 = 10.0;
pub const TOUCH_EASING_DISTANCE_THRESHOLD: f32 = 1.0;
pub const MAX_TOUCH_MOVEMENT_SPEED: f32 = 100.0;
pub const TOUCH_EASING_FACTOR: f32 = 1.5;

#[derive(Resource, Default, Debug, Copy, Clone)]
pub enum CursorPosition {
    #[default]
    None,
    Some {
        position: Vec2,
        remaining_time: f32,
    },
}

#[derive(Resource, Default, Debug, Clone)]
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

/// Calculates the primary direction from a 2D vector delta
pub fn calculate_direction_from_delta(delta: Vec2) -> Direction {
    if delta.x.abs() > delta.y.abs() {
        if delta.x > 0.0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if delta.y > 0.0 {
        Direction::Down
    } else {
        Direction::Up
    }
}

/// Updates the touch reference position with easing
///
/// This slowly moves the start_pos towards the current_pos, with the speed
/// decreasing as the distance gets smaller. The maximum movement speed is capped.
/// Returns the delta vector and its length for reuse by the caller.
pub fn update_touch_reference_position(touch_data: &mut TouchData, delta_time: f32) -> (Vec2, f32) {
    // Calculate the vector from start to current position
    let delta = touch_data.current_pos - touch_data.start_pos;
    let distance = delta.length();

    // If there's no significant distance, nothing to do
    if distance < TOUCH_EASING_DISTANCE_THRESHOLD {
        return (delta, distance);
    }

    // Calculate speed based on distance (slower as it gets closer)
    // The easing function creates a curve where movement slows down as it approaches the target
    let speed = (distance / TOUCH_EASING_FACTOR).min(MAX_TOUCH_MOVEMENT_SPEED);

    // Calculate movement distance for this frame
    let movement_amount = speed * delta_time;

    // If the movement would overshoot, just set to target
    if movement_amount >= distance {
        touch_data.start_pos = touch_data.current_pos;
    } else {
        // Use direct vector scaling instead of normalization
        let scale_factor = movement_amount / distance;
        touch_data.start_pos += delta * scale_factor;
    }

    (delta, distance)
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
                    touch_data.current_pos = Vec2::new(x as f32, y as f32);
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
                        touch_data.current_pos = Vec2::new(screen_x, screen_y);
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
            Event::Window { win_event, .. } => {
                if let WindowEvent::Resized(w, h) = win_event {
                    tracing::info!(width = w, height = h, event = ?win_event, "Window Resized");
                }
            }
            // Despite disabling this event, it's still received, so we ignore it explicitly.
            Event::RenderTargetsReset { .. } => {}
            _ => {
                tracing::warn!(event = ?event, "Unhandled Event");
            }
        }
    }

    // Delegate keyboard handling to shared logic used by tests and production.
    let emitted = process_simple_key_events(&mut bindings, &simple_key_events);
    for event in emitted {
        writer.write(event);
    }

    // Update touch reference position with easing
    if let Some(ref mut touch_data) = touch_state.active_touch {
        // Apply easing to the reference position and get the delta for direction calculation
        let (delta, distance) = update_touch_reference_position(touch_data, delta_time.seconds);

        // Check for direction based on updated reference position
        if distance >= TOUCH_DIRECTION_THRESHOLD {
            let direction = calculate_direction_from_delta(delta);

            // Only send command if direction has changed
            if touch_data.current_direction != Some(direction) {
                touch_data.current_direction = Some(direction);
                writer.write(GameEvent::Command(GameCommand::MovePlayer(direction)));
            }
        } else if touch_data.current_direction.is_some() {
            touch_data.current_direction = None;
        }
    }

    if let (false, CursorPosition::Some { remaining_time, .. }) = (cursor_seen, &mut *cursor) {
        *remaining_time -= delta_time.seconds;
        if *remaining_time <= 0.0 {
            *cursor = CursorPosition::None;
        }
    }
}
