//! System schedule configuration and execution ordering.

use std::ops::Not;

use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, SystemSet};
use bevy_ecs::system::{Local, Res, ResMut};

use crate::systems;
use crate::systems::animation::{blinking_system, directional_render_system, linear_render_system};
use crate::systems::audio::audio_system;
use crate::systems::collision::collision_system;
use crate::systems::common::ScoreResource;
use crate::systems::ghost::{eaten_ghost_system, ghost_movement_system, ghost_state_system};
use crate::systems::hud::{
    fruit_sprite_system, hud_render_system, player_life_sprite_system, touch_ui_render_system, FruitSprites,
};
use crate::systems::lifetime::time_to_live_system;
use crate::systems::profiling::{profile, SystemId};
use crate::systems::render::{combined_render_system, dirty_render_system, present_system, RenderDirty};
use crate::systems::state::GameStage;
use crate::systems::state::PauseState;

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

pub(super) fn configure_schedule(schedule: &mut Schedule) {
    let stage_system = profile(SystemId::Stage, systems::state::stage_system);
    let input_system = profile(SystemId::Input, systems::input::input_system);
    let pause_system = profile(SystemId::Input, systems::state::handle_pause_command);
    let player_control_system = profile(SystemId::PlayerControls, systems::player::player_control_system);
    let player_movement_system = profile(SystemId::PlayerMovement, systems::player::player_movement_system);
    let player_tunnel_slowdown_system = profile(SystemId::PlayerMovement, systems::player::player_tunnel_slowdown_system);
    let ghost_mode_tick_system = profile(SystemId::Ghost, systems::ghost::ghost_mode_tick_system);
    let ghost_house_system = profile(SystemId::Ghost, systems::ghost::ghost_house_system);
    let elroy_system = profile(SystemId::Ghost, systems::ghost::elroy_system);
    let ghost_targeting_system = profile(SystemId::Ghost, systems::ghost::ghost_targeting_system);
    let ghost_movement_system = profile(SystemId::Ghost, ghost_movement_system);
    let collision_system = profile(SystemId::Collision, collision_system);
    let audio_system = profile(SystemId::Audio, audio_system);
    let blinking_system = profile(SystemId::Blinking, blinking_system);
    let directional_render_system = profile(SystemId::DirectionalRender, directional_render_system);
    let linear_render_system = profile(SystemId::LinearRender, linear_render_system);
    let dirty_render_system = profile(SystemId::DirtyRender, dirty_render_system);
    let hud_render_system = profile(SystemId::HudRender, hud_render_system);
    let player_life_sprite_system = profile(SystemId::HudRender, player_life_sprite_system);
    let fruit_sprite_system = profile(SystemId::HudRender, fruit_sprite_system);
    let present_system = profile(SystemId::Present, present_system);
    let unified_ghost_state_system = profile(SystemId::GhostStateAnimation, ghost_state_system);
    let eaten_ghost_system = profile(SystemId::EatenGhost, eaten_ghost_system);
    let time_to_live_system = profile(SystemId::TimeToLive, time_to_live_system);
    let manage_pause_state_system = profile(SystemId::PauseManager, systems::state::manage_pause_state_system);

    // Input system should always run to prevent SDL event pump from blocking
    let input_systems = (
        input_system.run_if(|mut local: Local<u8>| {
            *local = local.wrapping_add(1u8);
            // run every nth frame
            *local % 2 == 0
        }),
        player_control_system,
        pause_system,
        #[cfg(not(target_os = "emscripten"))]
        profile(SystemId::Input, systems::state::handle_fullscreen_command),
    )
        .chain();

    schedule
        .add_systems((
            input_systems.in_set(GameplaySet::Input),
            time_to_live_system.before(GameplaySet::Update),
            (
                (
                    player_movement_system,
                    player_tunnel_slowdown_system,
                    ghost_mode_tick_system,
                    ghost_house_system,
                    elroy_system,
                    ghost_targeting_system,
                    ghost_movement_system,
                    eaten_ghost_system,
                )
                    .chain(),
                collision_system,
                unified_ghost_state_system,
            )
                .chain()
                .in_set(GameplaySet::Update),
            (
                blinking_system,
                directional_render_system,
                linear_render_system,
                player_life_sprite_system,
                fruit_sprite_system.run_if(|f: Res<FruitSprites>| f.is_changed()),
            )
                .in_set(RenderSet::Animation),
            stage_system.in_set(GameplaySet::Respond),
            (
                (|mut dirty: ResMut<RenderDirty>, score: Res<ScoreResource>, stage: Res<GameStage>| {
                    dirty.0 |= score.is_changed() || stage.is_changed();
                }),
                dirty_render_system.run_if(|dirty: Res<RenderDirty>| dirty.0.not()),
                combined_render_system,
                hud_render_system,
                touch_ui_render_system,
            )
                .chain()
                .in_set(RenderSet::Draw),
            (present_system, audio_system).chain().in_set(RenderSet::Present),
            manage_pause_state_system.after(GameplaySet::Update),
        ))
        .configure_sets(
            (
                GameplaySet::Input,
                GameplaySet::Update.run_if(|paused: Res<PauseState>| !paused.active()),
                GameplaySet::Respond.run_if(|paused: Res<PauseState>| !paused.active()),
                RenderSet::Animation.run_if(|paused: Res<PauseState>| !paused.active()),
                RenderSet::Draw,
                RenderSet::Present,
            )
                .chain(),
        );
}
