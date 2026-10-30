use assert2::check;
use pacman::systems::ghost::mode::{GhostModeController, ScatterChaseMode};
#[test]
fn new_starts_in_scatter() {
    let ctrl = GhostModeController::new(1);
    check!(ctrl.mode == ScatterChaseMode::Scatter);
}

#[test]
fn new_starts_at_phase_0() {
    let ctrl = GhostModeController::new(1);
    check!(ctrl.phase_index == 0);
}

#[test]
fn new_not_paused() {
    let ctrl = GhostModeController::new(1);
    check!(!ctrl.paused);
}

#[test]
fn default_is_level_1() {
    let ctrl = GhostModeController::default();
    check!(ctrl.level == 1);
    check!(ctrl.mode == ScatterChaseMode::Scatter);
}
#[test]
fn level_1_initial_scatter_7_seconds() {
    let ctrl = GhostModeController::new(1);
    // Level 1: 7 * 60 = 420 ticks
    check!(ctrl.mode_timer == 7 * 60);
}

#[test]
fn level_2_initial_scatter_7_seconds() {
    let ctrl = GhostModeController::new(2);
    check!(ctrl.mode_timer == 7 * 60);
}

#[test]
fn level_5_initial_scatter_5_seconds() {
    let ctrl = GhostModeController::new(5);
    check!(ctrl.mode_timer == 5 * 60);
}
#[test]
fn tick_decrements_timer() {
    let mut ctrl = GhostModeController::new(1);
    let initial = ctrl.mode_timer;
    let changed = ctrl.tick();
    check!(!changed);
    check!(ctrl.mode_timer == initial - 1);
}

#[test]
fn tick_transitions_scatter_to_chase() {
    let mut ctrl = GhostModeController::new(1);
    let scatter_duration = ctrl.mode_timer;
    // Tick through all scatter ticks (timer decrements to 0)
    for _ in 0..scatter_duration {
        check!(!ctrl.tick());
    }
    // Transition happens on the tick after timer reaches 0
    check!(ctrl.tick());
    check!(ctrl.mode == ScatterChaseMode::Chase);
    check!(ctrl.phase_index == 1);
}

#[test]
fn tick_transitions_chase_to_scatter() {
    let mut ctrl = GhostModeController::new(1);
    // Tick through phase 0 (scatter 7s = 420 ticks + 1 transition tick)
    let scatter_duration = ctrl.mode_timer;
    for _ in 0..scatter_duration {
        check!(!ctrl.tick());
    }
    check!(ctrl.tick());
    check!(ctrl.mode == ScatterChaseMode::Chase);

    // Tick through phase 1 (chase 20s = 1200 ticks + 1 transition tick)
    let chase_duration = ctrl.mode_timer;
    for _ in 0..chase_duration {
        check!(!ctrl.tick());
    }
    check!(ctrl.tick());
    check!(ctrl.mode == ScatterChaseMode::Scatter);
    check!(ctrl.phase_index == 2);
}
#[test]
fn paused_timer_doesnt_decrement() {
    let mut ctrl = GhostModeController::new(1);
    ctrl.pause();
    let timer_before = ctrl.mode_timer;
    let changed = ctrl.tick();
    check!(!changed);
    check!(ctrl.mode_timer == timer_before);
}

#[test]
fn resume_allows_ticking() {
    let mut ctrl = GhostModeController::new(1);
    ctrl.pause();
    ctrl.tick();
    ctrl.resume();
    let timer_before = ctrl.mode_timer;
    ctrl.tick();
    check!(ctrl.mode_timer == timer_before - 1);
}
#[test]
fn indefinite_phase_never_transitions() {
    let mut ctrl = GhostModeController::new(1);
    // Tick through all 7 phases to reach the indefinite chase (phase 7)
    // Level 1 phases: S(420), C(1200), S(420), C(1200), S(300), C(1200), S(300), C(indefinite)
    for expected_phase in 0..7 {
        check!(ctrl.phase_index == expected_phase);
        let duration = ctrl.mode_timer;
        for _ in 0..duration {
            check!(!ctrl.tick());
        }
        check!(ctrl.tick()); // transition to next phase
    }

    // Now at phase 7 -- indefinite chase with u32::MAX sentinel
    check!(ctrl.phase_index == 7);
    check!(ctrl.mode == ScatterChaseMode::Chase);
    check!(ctrl.mode_timer == u32::MAX);

    // u32::MAX sentinel prevents any timer decrement -- a single tick suffices to verify
    check!(!ctrl.tick());
    check!(ctrl.mode_timer == u32::MAX);
}
#[test]
fn reset_restores_initial_state() {
    let mut ctrl = GhostModeController::new(1);
    // Mutate it
    ctrl.phase_index = 3;
    ctrl.mode = ScatterChaseMode::Chase;
    ctrl.mode_timer = 42;
    ctrl.paused = true;

    ctrl.reset(5);
    check!(ctrl.mode == ScatterChaseMode::Scatter);
    check!(ctrl.phase_index == 0);
    check!(!ctrl.paused);
    check!(ctrl.level == 5);
    check!(ctrl.mode_timer == 5 * 60);
}
