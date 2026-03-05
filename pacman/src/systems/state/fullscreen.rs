#[cfg(not(target_os = "emscripten"))]
use bevy_ecs::{event::EventReader, system::NonSendMut};
#[cfg(not(target_os = "emscripten"))]
use sdl2::video::FullscreenType;
#[cfg(not(target_os = "emscripten"))]
use tracing::info;

#[cfg(not(target_os = "emscripten"))]
use crate::events::{GameCommand, GameEvent};
#[cfg(not(target_os = "emscripten"))]
use crate::systems::render::CanvasResource;

#[cfg(not(target_os = "emscripten"))]
pub fn handle_fullscreen_command(mut events: EventReader<GameEvent>, mut canvas: NonSendMut<CanvasResource>) {
    for event in events.read() {
        if let GameEvent::Command(GameCommand::ToggleFullscreen) = event {
            let window = canvas.window_mut();
            let current = window.fullscreen_state();
            let target = match current {
                FullscreenType::Off => FullscreenType::Desktop,
                _ => FullscreenType::Off,
            };

            if let Err(e) = window.set_fullscreen(target) {
                tracing::warn!(error = ?e, "Failed to toggle fullscreen");
            } else {
                let on = matches!(target, FullscreenType::Desktop | FullscreenType::True);
                info!(fullscreen = on, "Toggled fullscreen");
            }
        }
    }
}
