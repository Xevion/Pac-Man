use assert2::check;
use pacman::systems::ghost::state::{BounceDirection, FrightenedData, GhostAnimationState, GhostState, HousePosition};
#[test]
fn in_house_is_in_house() {
    let state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    check!(state.is_in_house());
}

#[test]
fn exiting_is_in_house() {
    let state = GhostState::Exiting { progress: 0.5 };
    check!(state.is_in_house());
}

#[test]
fn reviving_is_in_house() {
    let state = GhostState::Reviving { remaining_ticks: 10 };
    check!(state.is_in_house());
}

#[test]
fn active_is_not_in_house() {
    let state = GhostState::Active { frightened: None };
    check!(!state.is_in_house());
}

#[test]
fn eyes_is_not_in_house() {
    let state = GhostState::Eyes;
    check!(!state.is_in_house());
}

#[test]
fn active_is_active() {
    let state = GhostState::Active { frightened: None };
    check!(state.is_active());
}

#[test]
fn active_frightened_is_active() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    check!(state.is_active());
}

#[test]
fn in_house_is_not_active() {
    let state = GhostState::InHouse {
        position: HousePosition::Left,
        bounce: BounceDirection::Down,
    };
    check!(!state.is_active());
}

#[test]
fn eyes_is_not_active() {
    let state = GhostState::Eyes;
    check!(!state.is_active());
}

#[test]
fn active_frightened_is_frightened() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    check!(state.is_frightened());
}

#[test]
fn active_not_frightened_is_not_frightened() {
    let state = GhostState::Active { frightened: None };
    check!(!state.is_frightened());
}

#[test]
fn in_house_is_not_frightened() {
    let state = GhostState::InHouse {
        position: HousePosition::Right,
        bounce: BounceDirection::Up,
    };
    check!(!state.is_frightened());
}

#[test]
fn eyes_is_not_frightened() {
    let state = GhostState::Eyes;
    check!(!state.is_frightened());
}
#[test]
fn in_house_animation_state_is_normal() {
    let state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    check!(state.animation_state() == GhostAnimationState::Normal);
}

#[test]
fn exiting_animation_state_is_normal() {
    let state = GhostState::Exiting { progress: 0.3 };
    check!(state.animation_state() == GhostAnimationState::Normal);
}

#[test]
fn active_no_fright_animation_state_is_normal() {
    let state = GhostState::Active { frightened: None };
    check!(state.animation_state() == GhostAnimationState::Normal);
}

#[test]
fn active_frightened_animation_state_no_flash() {
    let state = GhostState::Active {
        frightened: Some(FrightenedData::new(100, 50)),
    };
    check!(state.animation_state() == GhostAnimationState::Frightened { flash: false });
}

#[test]
fn active_frightened_flashing_animation_state() {
    let mut data = FrightenedData::new(100, 2);
    // Tick twice so flash_timer reaches 0 and flashing becomes true
    data.tick();
    data.tick();
    let state = GhostState::Active { frightened: Some(data) };
    check!(state.animation_state() == GhostAnimationState::Frightened { flash: true });
}

#[test]
fn eyes_animation_state_is_eyes() {
    let state = GhostState::Eyes;
    check!(state.animation_state() == GhostAnimationState::Eyes);
}

#[test]
fn reviving_animation_state_is_eyes() {
    let state = GhostState::Reviving { remaining_ticks: 5 };
    check!(state.animation_state() == GhostAnimationState::Eyes);
}
#[test]
fn tick_active_no_fright_returns_false() {
    let mut state = GhostState::Active { frightened: None };
    check!(!state.tick());
}

#[test]
fn tick_frightened_decrements_and_returns_false() {
    let mut state = GhostState::Active {
        frightened: Some(FrightenedData::new(10, 5)),
    };
    let changed = state.tick();
    check!(!changed);
    // Still frightened
    check!(state.is_frightened());
}

#[test]
fn tick_frightened_expires_transitions_to_active() {
    let mut state = GhostState::Active {
        frightened: Some(FrightenedData::new(1, 0)),
    };
    // First tick: remaining goes from 1 to 0, not expired yet
    let changed1 = state.tick();
    check!(!changed1);
    // Second tick: remaining is 0, expires
    let changed2 = state.tick();
    check!(changed2);
    check!(state == GhostState::Active { frightened: None });
}

#[test]
fn tick_reviving_decrements() {
    let mut state = GhostState::Reviving { remaining_ticks: 3 };
    let changed1 = state.tick();
    check!(!changed1);
    let changed2 = state.tick();
    check!(!changed2);
    // Tick 3: remaining goes from 1 to 0, signals change
    let changed3 = state.tick();
    check!(changed3);
}

#[test]
fn tick_reviving_zero_returns_false() {
    let mut state = GhostState::Reviving { remaining_ticks: 0 };
    check!(!state.tick());
}

#[test]
fn tick_in_house_returns_false() {
    let mut state = GhostState::InHouse {
        position: HousePosition::Center,
        bounce: BounceDirection::Up,
    };
    check!(!state.tick());
}

#[test]
fn tick_eyes_returns_false() {
    let mut state = GhostState::Eyes;
    check!(!state.tick());
}

#[test]
fn tick_exiting_returns_false() {
    let mut state = GhostState::Exiting { progress: 0.5 };
    check!(!state.tick());
}
#[test]
fn frightened_data_new() {
    let data = FrightenedData::new(120, 60);
    check!(data.remaining_ticks == 120);
    check!(!data.flashing);
    check!(data.flash_timer == 60);
}

#[test]
fn frightened_data_tick_decrements() {
    let mut data = FrightenedData::new(10, 5);
    let ended = data.tick();
    check!(!ended);
    check!(data.remaining_ticks == 9);
    check!(data.flash_timer == 4);
}

#[test]
fn frightened_data_flash_triggers_when_timer_hits_zero() {
    let mut data = FrightenedData::new(100, 2);
    check!(!data.flashing);
    data.tick(); // flash_timer: 2 -> 1
    check!(!data.flashing);
    data.tick(); // flash_timer: 1 -> 0, flashing = true
    check!(data.flashing);
}

#[test]
fn frightened_data_tick_returns_true_when_expired() {
    let mut data = FrightenedData::new(0, 0);
    let ended = data.tick();
    check!(ended);
}
#[test]
fn ghost_state_default_is_active_no_fright() {
    let state = GhostState::default();
    check!(state == GhostState::Active { frightened: None });
}
