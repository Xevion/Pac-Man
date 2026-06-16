use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    event::EventWriter,
    observer::Trigger,
    resource::Resource,
    schedule::SystemSet,
    system::{Commands, NonSendMut, Res, ResMut},
};
use glam::{UVec2, Vec2};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    EventPump,
};
use smallvec::{smallvec, SmallVec};

use crate::systems::common::{DeltaTime, GlobalState};
use crate::systems::layout::Layout;
use crate::{
    events::{ExitRequested, GameCommand, GameEvent},
    map::direction::Direction,
};

/// Ordering phases within the per-frame input set.
///
/// Systems that react to this frame's input must run after the pump drain, but the
/// schedule wraps every system in an opaque profiling closure, so they can't be ordered
/// with `.after(input_system)` by name. Instead the drain lives in [`InputSet::Drain`]
/// and everything reacting to it lives in [`InputSet::React`] (ordered after Drain),
/// which any module -- including the scene handlers -- can attach to by set label.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum InputSet {
    /// Drains the SDL event pump and emits this frame's `GameEvent`s and `HumanInput`.
    Drain,
    /// Reacts to this frame's input: movement production/consumption, scene control,
    /// and global commands.
    React,
}

/// Who is currently driving Pac-Man's movement. This gates only the `MovePlayer`
/// *producer*: [`input_system`] emits human movement under [`InputSource::Human`],
/// while [`ai_player_system`](crate::systems::ai::ai_player_system) emits under
/// [`InputSource::Ai`]. The single consumer (`player_control_system`) is unaware of
/// the source, so swapping who steers never touches the movement pipeline itself.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    #[default]
    Human,
    Ai,
}

impl InputSource {
    pub fn is_human(self) -> bool {
        matches!(self, InputSource::Human)
    }

    pub fn is_ai(self) -> bool {
        matches!(self, InputSource::Ai)
    }
}

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

/// Set by [`input_system`] every frame: true when a human pressed a key or began a
/// click/tap this frame, independent of [`InputSource`]. Computed from the raw SDL
/// events, which the AI never produces, so the attract demo can detect a human's intent
/// to start a real game even though AI mode suppresses human movement commands.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct HumanInput {
    pub active: bool,
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
        key_bindings.insert(Keycode::Escape, GameCommand::TogglePause);
        key_bindings.insert(Keycode::Space, GameCommand::ToggleDebug);
        key_bindings.insert(Keycode::M, GameCommand::MuteAudio);
        key_bindings.insert(Keycode::R, GameCommand::ResetLevel);
        key_bindings.insert(Keycode::T, GameCommand::SingleTick);

        #[cfg(not(target_os = "emscripten"))]
        {
            key_bindings.insert(Keycode::Q, GameCommand::Exit);
            // Desktop-only fullscreen toggle
            key_bindings.insert(Keycode::F, GameCommand::ToggleFullscreen);
        }

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

/// Browser-requested canvas size awaiting application, packed as
/// `(width << 32) | height`; `0` means nothing is pending. Written by the
/// `pacman_resize` FFI export (from a JS `ResizeObserver`) and consumed by
/// [`apply_pending_resize`]. WASM is single-threaded, so `Relaxed` suffices.
#[cfg(target_os = "emscripten")]
pub static PENDING_CANVAS_SIZE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Records a browser-requested canvas size for the next frame to apply.
#[cfg(target_os = "emscripten")]
pub fn request_canvas_resize(width: u32, height: u32) {
    let packed = ((width as u64) << 32) | height as u64;
    PENDING_CANVAS_SIZE.store(packed, std::sync::atomic::Ordering::Relaxed);
}

/// Applies a browser-requested canvas size by resizing the SDL window, which makes
/// SDL emit a `SizeChanged` event that [`input_system`] consumes to recompute the
/// [`Layout`]. Runs before `input_system` so the new size lands the same frame.
///
/// On Emscripten SDL only watches browser-*window* resizes, not CSS/flex layout
/// changes to the canvas, so this FFI-driven path is the sole resize signal on the
/// web. The drawable size is already in physical pixels (the JS side scaled by the
/// device pixel ratio), so no high-DPI handling is needed here.
#[cfg(target_os = "emscripten")]
pub fn apply_pending_resize(mut canvas: NonSendMut<crate::systems::render::CanvasResource>) {
    let packed = PENDING_CANVAS_SIZE.swap(0, std::sync::atomic::Ordering::Relaxed);
    if packed == 0 {
        return;
    }
    let (width, height) = ((packed >> 32) as u32, packed as u32);
    if canvas.window().size() == (width, height) {
        return;
    }
    if let Err(e) = canvas.window_mut().set_size(width, height) {
        tracing::warn!(error = ?e, width, height, "Failed to apply browser canvas resize");
    }
}

/// Observer for [`ExitRequested`]: flips the global exit flag so the main loop tears
/// down after the current frame. Raised by the quit key and the window close button,
/// it resolves the same frame rather than waiting on a buffered-event read.
pub fn exit_observer(_: Trigger<ExitRequested>, mut state: ResMut<GlobalState>) {
    state.exit = true;
}

#[allow(clippy::too_many_arguments)]
pub fn input_system(
    delta_time: Res<DeltaTime>,
    input_source: Res<InputSource>,
    mut commands: Commands,
    mut bindings: ResMut<Bindings>,
    mut writer: EventWriter<GameEvent>,
    mut pump: NonSendMut<EventPump>,
    mut cursor: ResMut<CursorPosition>,
    mut touch_state: ResMut<TouchState>,
    mut human_input: ResMut<HumanInput>,
    mut layout: ResMut<Layout>,
) {
    let mut cursor_seen = false;
    // Collect all events for this frame.
    let frame_events: SmallVec<[Event; 3]> = pump.poll_iter().collect();

    // A human acted this frame iff a key went down or a click/tap began. Derived from
    // the raw SDL events (which the AI never generates), so it stays a human-only signal
    // the attract demo can use to start a real game.
    human_input.active = frame_events.iter().any(|event| {
        matches!(
            event,
            Event::KeyDown {
                repeat: false,
                keycode: Some(_),
                ..
            } | Event::MouseButtonDown { .. }
                | Event::FingerDown { .. }
        )
    });

    // Handle non-keyboard events inline and build a simplified keyboard event stream.
    let mut simple_key_events: SmallVec<[SimpleKeyEvent; 3]> = smallvec![];
    for event in &frame_events {
        match *event {
            Event::Quit { .. } => {
                commands.trigger(ExitRequested);
            }
            Event::MouseMotion { x, y, .. } => {
                let pos = layout.window_to_maze(Vec2::new(x as f32, y as f32));
                *cursor = CursorPosition::Some {
                    position: pos,
                    remaining_time: 0.20,
                };
                cursor_seen = true;

                // Mouse doubles as touch for desktop testing.
                if let Some(ref mut touch_data) = touch_state.active_touch {
                    touch_data.current_pos = pos;
                }
            }
            // Handle mouse events as touch for desktop testing
            Event::MouseButtonDown { x, y, .. } => {
                let pos = layout.window_to_maze(Vec2::new(x as f32, y as f32));
                touch_state.active_touch = Some(TouchData::new(0, pos)); // ID 0 for mouse
            }
            Event::MouseButtonUp { .. } => {
                touch_state.active_touch = None;
            }
            // Handle actual touch events for mobile
            Event::FingerDown { finger_id, x, y, .. } => {
                // Touch arrives normalized (0..1) over the whole window; map it
                // through the layout onto the maze.
                let win = layout.window.as_vec2();
                let pos = layout.window_to_maze(Vec2::new(x * win.x, y * win.y));
                touch_state.active_touch = Some(TouchData::new(finger_id, pos));
            }
            Event::FingerMotion { finger_id, x, y, .. } => {
                if let Some(ref mut touch_data) = touch_state.active_touch {
                    if touch_data.finger_id == finger_id {
                        let win = layout.window.as_vec2();
                        touch_data.current_pos = layout.window_to_maze(Vec2::new(x * win.x, y * win.y));
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
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) | WindowEvent::SizeChanged(w, h) => {
                    // SDL fires both Resized and SizeChanged for a single user drag step;
                    // only recompute when the size truly changed so duplicate / no-op
                    // events don't re-dirty the frame and force a redundant re-render.
                    let size = UVec2::new(w.max(0) as u32, h.max(0) as u32);
                    if size != layout.window {
                        tracing::info!(width = size.x, height = size.y, "Window resized");
                        crate::tracy::message(&format!("resize {}x{}", size.x, size.y));
                        let _zone = tracing::debug_span!("layout_compute").entered();
                        *layout = Layout::compute(size);
                    }
                }
                _ => {}
            },
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
        match event {
            // Exit is a one-shot lifecycle request: route it to its observer instead
            // of the buffered command stream so shutdown happens the same frame.
            // (Desktop only -- the web build has no in-engine quit command.)
            #[cfg(not(target_os = "emscripten"))]
            GameEvent::Command(GameCommand::Exit) => commands.trigger(ExitRequested),
            // In AI-driven scenes (attract) the AI is the sole movement producer;
            // drop human movement so the two don't fight over the buffered direction.
            GameEvent::Command(GameCommand::MovePlayer(_)) if input_source.is_ai() => {}
            _ => {
                writer.write(event);
            }
        }
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
                if input_source.is_human() {
                    writer.write(GameEvent::Command(GameCommand::MovePlayer(direction)));
                }
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
