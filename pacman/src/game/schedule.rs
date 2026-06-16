//! System schedule configuration and execution ordering.

use std::ops::Not;

use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, SystemSet};
use bevy_ecs::system::{Res, ResMut};

use crate::scenes;
use crate::systems;
use crate::systems::animation::{blinking_system, directional_render_system, linear_render_system};
use crate::systems::audio::audio_system;
use crate::systems::collision::collision_system;
use crate::systems::debug::debug_overlay_system;
use crate::systems::ghost::{eaten_ghost_system, ghost_movement_system, ghost_state_system};
use crate::systems::hud::{chrome_render_system, hud_overlay_system, touch_ui_render_system};
use crate::systems::lifetime::time_to_live_system;
use crate::systems::profiling::{profile, SystemId};
use crate::systems::render::{backbuffer_render_system, composite_maze_system, dirty_render_system, present_system, RenderDirty};
use crate::systems::state::PauseState;
use crate::systems::state::Session;

/// System set for all gameplay systems to ensure they run after input processing
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameplaySet {
    /// Gameplay systems that process inputs
    Input,
    /// Gameplay systems that update the game state
    Update,
    /// Gameplay systems that respond to events
    Respond,
}

/// System set for all rendering systems to ensure they run after gameplay logic
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum RenderSet {
    Animation,
    Draw,
    Present,
}

/// Builds the full system schedule. Each set is wired by its own helper so the
/// ordering for a given phase lives in one place; `configure_sets` then chains
/// the sets into their run order.
pub(super) fn configure_schedule(schedule: &mut Schedule) {
    // Scene transitions are applied before anything else runs, so a freshly entered
    // scene's entities exist for the rest of the frame. This is an exclusive system
    // (it spawns/despawns whole populations) and so sits outside the parallel sets.
    schedule.add_systems(scenes::apply_pending_scene.before(GameplaySet::Input));

    add_input_systems(schedule);
    add_update_systems(schedule);
    add_respond_systems(schedule);
    add_animation_systems(schedule);
    add_draw_systems(schedule);
    add_present_systems(schedule);

    // The simulation sets run only inside a live gameplay scene that isn't paused.
    // Input, Draw, and Present run every frame regardless of scene (input must drain
    // the SDL pump; the title/menu still needs to render and present).
    schedule.configure_sets(
        (
            GameplaySet::Input,
            GameplaySet::Update.run_if(scenes::sim_active),
            GameplaySet::Respond.run_if(scenes::sim_active),
            RenderSet::Animation.run_if(scenes::sim_active),
            RenderSet::Draw,
            RenderSet::Present,
        )
            .chain(),
    );
}

/// Input set. Must run every frame: it is the sole drain of the SDL event pump, so
/// skipping a frame leaves events (including window resizes) queued. An undrained
/// pump reads to the OS as an unresponsive window and makes resizing feel laggy.
fn add_input_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            // On the web, apply any browser-requested canvas resize before draining the
            // event pump so the resulting SizeChanged event lands the same frame.
            #[cfg(target_os = "emscripten")]
            profile(SystemId::Input, systems::input::apply_pending_resize),
            profile(SystemId::Input, systems::input::input_system),
            // While the Title is up, the first input hands off to gameplay. Ordered
            // after input_system so it sees this frame's GameEvents.
            profile(SystemId::Input, scenes::title_input_system).run_if(scenes::in_scene(scenes::Scene::Title)),
            profile(SystemId::PlayerControls, systems::player::player_control_system),
            profile(SystemId::Input, systems::state::handle_pause_command),
            #[cfg(not(target_os = "emscripten"))]
            profile(SystemId::Input, systems::state::handle_fullscreen_command),
        )
            .chain()
            .in_set(GameplaySet::Input),
    );
}

/// Update set plus the lifetime/pause systems ordered around it.
fn add_update_systems(schedule: &mut Schedule) {
    schedule.add_systems((
        profile(SystemId::TimeToLive, time_to_live_system).before(GameplaySet::Update),
        (
            (
                profile(SystemId::PlayerMovement, systems::player::player_movement_system),
                profile(SystemId::PlayerMovement, systems::player::player_tunnel_slowdown_system),
                profile(SystemId::Ghost, systems::ghost::ghost_mode_tick_system),
                profile(SystemId::Ghost, systems::ghost::ghost_house_system),
                profile(SystemId::Ghost, systems::ghost::elroy_system),
                profile(SystemId::Ghost, systems::ghost::ghost_targeting_system),
                profile(SystemId::Ghost, ghost_movement_system),
                profile(SystemId::EatenGhost, eaten_ghost_system),
            )
                .chain(),
            profile(SystemId::Collision, collision_system),
            profile(SystemId::GhostStateAnimation, ghost_state_system),
        )
            .chain()
            .in_set(GameplaySet::Update),
        profile(SystemId::PauseManager, systems::state::manage_pause_state_system).after(GameplaySet::Update),
    ));
}

/// Respond set: event-driven stage transitions.
fn add_respond_systems(schedule: &mut Schedule) {
    schedule.add_systems(profile(SystemId::Stage, systems::state::stage_system).in_set(GameplaySet::Respond));
}

/// Animation set: sprite frame selection (unordered within the set).
fn add_animation_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            profile(SystemId::Blinking, blinking_system),
            profile(SystemId::DirectionalRender, directional_render_system),
            profile(SystemId::LinearRender, linear_render_system),
        )
            .in_set(RenderSet::Animation),
    );
}

/// Draw set: dirty-tracking gate, backbuffer render, and HUD overlays.
fn add_draw_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            (|mut dirty: ResMut<RenderDirty>, session: Res<Session>, pause: Res<PauseState>| {
                // Session is a superset trigger: it changes on score/lives/level/stage/
                // intro/pellet edits, all of which warrant a redraw. The per-frame ghost
                // controllers are separate resources, so they don't over-trigger this.
                dirty.0 |= session.is_changed() || pause.is_changed();
            }),
            profile(SystemId::DirtyRender, dirty_render_system).run_if(|dirty: Res<RenderDirty>| dirty.0.not()),
            profile(SystemId::Render, backbuffer_render_system),
            // Maze-overlay text (READY!, GAME OVER, pause dimmer) is gameplay-specific,
            // so it stays off the Title's empty maze.
            profile(SystemId::HudRender, hud_overlay_system).run_if(scenes::in_scene(scenes::Scene::Gameplay)),
            touch_ui_render_system,
        )
            .chain()
            .in_set(RenderSet::Draw),
    );
}

/// Present set: composite the maze, draw window-space chrome, and present.
fn add_present_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            profile(SystemId::Present, composite_maze_system),
            profile(SystemId::HudRender, chrome_render_system),
            profile(SystemId::DebugRender, debug_overlay_system),
            profile(SystemId::Present, present_system),
            profile(SystemId::Audio, audio_system),
        )
            .chain()
            .in_set(RenderSet::Present),
    );
}
