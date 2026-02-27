use pacman::systems::ghost::state::{BounceDirection, FrightenedData, GhostAnimationState, GhostState, HousePosition};
use speculoos::prelude::*;
#[test]
fn in_house_is_in_house() {
    let state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    assert_that(&state.is_in_house()).is_true();
}

#[test]
fn exiting_is_in_house() {
    let state = GhostState::Exiting { progress: 0.5 };
    assert_that(&state.is_in_house()).is_true();
}

#[test]
fn reviving_is_in_house() {
    let state = GhostState::Reviving { remaining_ticks: 10 };
    assert_that(&state.is_in_house()).is_true();
}

#[test]
fn active_is_not_in_house() {
    let state = GhostState::Active { frightened: None };
    assert_that(&state.is_in_house()).is_false();
}

#[test]
fn eyes_is_not_in_house() {
    let state = GhostState::Eyes;
    assert_that(&state.is_in_house()).is_false();
}

#[test]
fn active_is_active() {
    let state = GhostState::Active { frightened: None };
    assert_that(&state.is_active()).is_true();
}

#[test]
fn active_frightened_is_active() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    assert_that(&state.is_active()).is_true();
}

#[test]
fn in_house_is_not_active() {
    let state = GhostState::InHouse {
        position: HousePosition::Left,
        bounce: BounceDirection::Down,
    };
    assert_that(&state.is_active()).is_false();
}

#[test]
fn eyes_is_not_active() {
    let state = GhostState::Eyes;
    assert_that(&state.is_active()).is_false();
}

#[test]
fn active_frightened_is_frightened() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    assert_that(&state.is_frightened()).is_true();
}

#[test]
fn active_not_frightened_is_not_frightened() {
    let state = GhostState::Active { frightened: None };
    assert_that(&state.is_frightened()).is_false();
}

#[test]
fn in_house_is_not_frightened() {
    let state = GhostState::InHouse {
        position: HousePosition::Right,
        bounce: BounceDirection::Up,
    };
    assert_that(&state.is_frightened()).is_false();
}

#[test]
fn eyes_is_not_frightened() {
    let state = GhostState::Eyes;
    assert_that(&state.is_frightened()).is_false();
}
#[test]
fn in_house_animation_state_is_normal() {
    let state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Normal);
}

#[test]
fn exiting_animation_state_is_normal() {
    let state = GhostState::Exiting { progress: 0.3 };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Normal);
}

#[test]
fn active_no_fright_animation_state_is_normal() {
    let state = GhostState::Active { frightened: None };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Normal);
}

#[test]
fn active_frightened_animation_state_no_flash() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Frightened { flash: false });
}

#[test]
fn active_frightened_flashing_animation_state() {
    let mut data = FrightenedData::new(100, 2);
    // Tick twice so flash_timer reaches 0 and flashing becomes true
    data.tick();
    data.tick();
    let state = GhostState::Active { frightened: Some(data) };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Frightened { flash: true });
}

#[test]
fn eyes_animation_state_is_eyes() {
    let state = GhostState::Eyes;
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Eyes);
}

#[test]
fn reviving_animation_state_is_eyes() {
    let state = GhostState::Reviving { remaining_ticks: 5 };
    assert_that(&state.animation_state()).is_equal_to(GhostAnimationState::Eyes);
}
#[test]
fn tick_active_no_fright_returns_false() {
    let mut state = GhostState::Active { frightened: None };
    assert_that(&state.tick()).is_false();
}

#[test]
fn tick_frightened_decrements_and_returns_false() {
    let mut state = GhostState::Active {
        frightened: Some(FrightenedData::new(10, 5)),
    };
    let changed = state.tick();
    assert_that(&changed).is_false();
    // Still frightened
    assert_that(&state.is_frightened()).is_true();
}

#[test]
fn tick_frightened_expires_transitions_to_active() {
    let mut state = GhostState::Active {
        frightened: Some(FrightenedData::new(1, 0)),
    };
    // First tick: remaining goes from 1 to 0, not expired yet
    let changed1 = state.tick();
    assert_that(&changed1).is_false();
    // Second tick: remaining is 0, expires
    let changed2 = state.tick();
    assert_that(&changed2).is_true();
    assert_that(&state).is_equal_to(GhostState::Active { frightened: None });
}

#[test]
fn tick_reviving_decrements() {
    let mut state = GhostState::Reviving { remaining_ticks: 3 };
    let changed1 = state.tick();
    assert_that(&changed1).is_false();
    let changed2 = state.tick();
    assert_that(&changed2).is_false();
    // Tick 3: remaining goes from 1 to 0, signals change
    let changed3 = state.tick();
    assert_that(&changed3).is_true();
}

#[test]
fn tick_reviving_zero_returns_false() {
    let mut state = GhostState::Reviving { remaining_ticks: 0 };
    assert_that(&state.tick()).is_false();
}

#[test]
fn tick_in_house_returns_false() {
    let mut state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    assert_that(&state.tick()).is_false();
}

#[test]
fn tick_eyes_returns_false() {
    let mut state = GhostState::Eyes;
    assert_that(&state.tick()).is_false();
}

#[test]
fn tick_exiting_returns_false() {
    let mut state = GhostState::Exiting { progress: 0.5 };
    assert_that(&state.tick()).is_false();
}
#[test]
fn frightened_data_new() {
    let data = FrightenedData::new(120, 60);
    assert_that(&data.remaining_ticks).is_equal_to(120);
    assert_that(&data.flashing).is_false();
    assert_that(&data.flash_timer).is_equal_to(60);
}

#[test]
fn frightened_data_tick_decrements() {
    let mut data = FrightenedData::new(10, 5);
    let ended = data.tick();
    assert_that(&ended).is_false();
    assert_that(&data.remaining_ticks).is_equal_to(9);
    assert_that(&data.flash_timer).is_equal_to(4);
}

#[test]
fn frightened_data_flash_triggers_when_timer_hits_zero() {
    let mut data = FrightenedData::new(100, 2);
    assert_that(&data.flashing).is_false();
    data.tick(); // flash_timer: 2 -> 1
    assert_that(&data.flashing).is_false();
    data.tick(); // flash_timer: 1 -> 0, flashing = true
    assert_that(&data.flashing).is_true();
}

#[test]
fn frightened_data_tick_returns_true_when_expired() {
    let mut data = FrightenedData::new(0, 0);
    let ended = data.tick();
    assert_that(&ended).is_true();
}
#[test]
fn ghost_state_default_is_active_no_fright() {
    let state = GhostState::default();
    assert_that(&state).is_equal_to(GhostState::Active { frightened: None });
}
