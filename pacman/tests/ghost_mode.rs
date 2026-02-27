use pacman::systems::ghost::mode::{GhostModeController, ScatterChaseMode};
use speculoos::prelude::*;
#[test]
fn new_starts_in_scatter() {
    let ctrl = GhostModeController::new(1);
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Scatter);
}

#[test]
fn new_starts_at_phase_0() {
    let ctrl = GhostModeController::new(1);
    assert_that(&ctrl.phase_index).is_equal_to(0);
}

#[test]
fn new_not_paused() {
    let ctrl = GhostModeController::new(1);
    assert_that(&ctrl.paused).is_false();
}

#[test]
fn default_is_level_1() {
    let ctrl = GhostModeController::default();
    assert_that(&ctrl.level).is_equal_to(1);
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Scatter);
}
#[test]
fn level_1_initial_scatter_7_seconds() {
    let ctrl = GhostModeController::new(1);
    // Level 1: 7 * 60 = 420 ticks
    assert_that(&ctrl.mode_timer).is_equal_to(7 * 60);
}

#[test]
fn level_2_initial_scatter_7_seconds() {
    let ctrl = GhostModeController::new(2);
    assert_that(&ctrl.mode_timer).is_equal_to(7 * 60);
}

#[test]
fn level_5_initial_scatter_5_seconds() {
    let ctrl = GhostModeController::new(5);
    assert_that(&ctrl.mode_timer).is_equal_to(5 * 60);
}
#[test]
fn tick_decrements_timer() {
    let mut ctrl = GhostModeController::new(1);
    let initial = ctrl.mode_timer;
    let changed = ctrl.tick();
    assert_that(&changed).is_false();
    assert_that(&ctrl.mode_timer).is_equal_to(initial - 1);
}

#[test]
fn tick_transitions_scatter_to_chase() {
    let mut ctrl = GhostModeController::new(1);
    let scatter_duration = ctrl.mode_timer;
    // Tick through all scatter ticks (timer decrements to 0)
    for _ in 0..scatter_duration {
        assert_that(&ctrl.tick()).is_false();
    }
    // Transition happens on the tick after timer reaches 0
    assert_that(&ctrl.tick()).is_true();
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Chase);
    assert_that(&ctrl.phase_index).is_equal_to(1);
}

#[test]
fn tick_transitions_chase_to_scatter() {
    let mut ctrl = GhostModeController::new(1);
    // Tick through phase 0 (scatter 7s = 420 ticks + 1 transition tick)
    let scatter_duration = ctrl.mode_timer;
    for _ in 0..scatter_duration {
        assert_that(&ctrl.tick()).is_false();
    }
    assert_that(&ctrl.tick()).is_true();
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Chase);

    // Tick through phase 1 (chase 20s = 1200 ticks + 1 transition tick)
    let chase_duration = ctrl.mode_timer;
    for _ in 0..chase_duration {
        assert_that(&ctrl.tick()).is_false();
    }
    assert_that(&ctrl.tick()).is_true();
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Scatter);
    assert_that(&ctrl.phase_index).is_equal_to(2);
}
#[test]
fn paused_timer_doesnt_decrement() {
    let mut ctrl = GhostModeController::new(1);
    ctrl.pause();
    let timer_before = ctrl.mode_timer;
    let changed = ctrl.tick();
    assert_that(&changed).is_false();
    assert_that(&ctrl.mode_timer).is_equal_to(timer_before);
}

#[test]
fn resume_allows_ticking() {
    let mut ctrl = GhostModeController::new(1);
    ctrl.pause();
    ctrl.tick();
    ctrl.resume();
    let timer_before = ctrl.mode_timer;
    ctrl.tick();
    assert_that(&ctrl.mode_timer).is_equal_to(timer_before - 1);
}
#[test]
fn indefinite_phase_never_transitions() {
    let mut ctrl = GhostModeController::new(1);
    // Tick through all 7 phases to reach the indefinite chase (phase 7)
    // Level 1 phases: S(420), C(1200), S(420), C(1200), S(300), C(1200), S(300), C(indefinite)
    for expected_phase in 0..7 {
        assert_that(&ctrl.phase_index).is_equal_to(expected_phase);
        let duration = ctrl.mode_timer;
        for _ in 0..duration {
            assert_that(&ctrl.tick()).is_false();
        }
        assert_that(&ctrl.tick()).is_true(); // transition to next phase
    }

    // Now at phase 7 -- indefinite chase with u32::MAX sentinel
    assert_that(&ctrl.phase_index).is_equal_to(7);
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Chase);
    assert_that(&ctrl.mode_timer).is_equal_to(u32::MAX);

    // u32::MAX sentinel prevents any timer decrement -- a single tick suffices to verify
    assert_that(&ctrl.tick()).is_false();
    assert_that(&ctrl.mode_timer).is_equal_to(u32::MAX);
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
    assert_that(&ctrl.mode).is_equal_to(ScatterChaseMode::Scatter);
    assert_that(&ctrl.phase_index).is_equal_to(0);
    assert_that(&ctrl.paused).is_false();
    assert_that(&ctrl.level).is_equal_to(5);
    assert_that(&ctrl.mode_timer).is_equal_to(5 * 60);
}
