use assert2::check;
use pacman::systems::ghost::GhostType;
use pacman::systems::ghost::house::GhostHouseController;
#[test]
fn blinky_always_exits() {
    let ctrl = GhostHouseController::new(1);
    check!(ctrl.should_exit(GhostType::Blinky));
}

#[test]
fn blinky_always_exits_any_level() {
    for level in 1..=20 {
        let ctrl = GhostHouseController::new(level);
        check!(ctrl.should_exit(GhostType::Blinky));
    }
}
#[test]
fn pinky_exits_immediately_level_1() {
    let ctrl = GhostHouseController::new(1);
    // Active counter starts at 0 (Pinky), dot limit for Pinky is 0
    check!(ctrl.should_exit(GhostType::Pinky));
}

#[test]
fn pinky_exits_immediately_level_5() {
    let ctrl = GhostHouseController::new(5);
    check!(ctrl.should_exit(GhostType::Pinky));
}
#[test]
fn inky_does_not_exit_initially_level_1() {
    let ctrl = GhostHouseController::new(1);
    // Active counter is 0 (Pinky), not 1 (Inky), so Inky can't exit
    check!(!ctrl.should_exit(GhostType::Inky));
}

#[test]
fn inky_exits_after_30_dots_level_1() {
    let mut ctrl = GhostHouseController::new(1);
    // Advance counter: Pinky exits -> active_counter becomes 1 (Inky)
    ctrl.on_ghost_exit(GhostType::Pinky);

    // Eat 30 dots
    for _ in 0..30 {
        ctrl.on_dot_eaten();
    }
    check!(ctrl.should_exit(GhostType::Inky));
}

#[test]
fn inky_does_not_exit_after_29_dots_level_1() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Pinky);

    for _ in 0..29 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Inky));
}

#[test]
fn inky_exits_immediately_level_2() {
    let mut ctrl = GhostHouseController::new(2);
    ctrl.on_ghost_exit(GhostType::Pinky);
    // Level 2: Inky's dot limit is 0
    check!(ctrl.should_exit(GhostType::Inky));
}
#[test]
fn clyde_does_not_exit_initially() {
    let ctrl = GhostHouseController::new(1);
    check!(!ctrl.should_exit(GhostType::Clyde));
}

#[test]
fn clyde_needs_60_dots_level_1() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Pinky);
    ctrl.on_ghost_exit(GhostType::Inky);

    for _ in 0..59 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Clyde));

    ctrl.on_dot_eaten(); // 60th dot
    check!(ctrl.should_exit(GhostType::Clyde));
}

#[test]
fn clyde_needs_50_dots_level_2() {
    let mut ctrl = GhostHouseController::new(2);
    ctrl.on_ghost_exit(GhostType::Pinky);
    ctrl.on_ghost_exit(GhostType::Inky);

    for _ in 0..49 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Clyde));

    ctrl.on_dot_eaten(); // 50th dot
    check!(ctrl.should_exit(GhostType::Clyde));
}

#[test]
fn clyde_exits_immediately_level_3() {
    let mut ctrl = GhostHouseController::new(3);
    ctrl.on_ghost_exit(GhostType::Pinky);
    ctrl.on_ghost_exit(GhostType::Inky);
    // Level 3+: Clyde's dot limit is 0
    check!(ctrl.should_exit(GhostType::Clyde));
}
#[test]
fn counter_advances_pinky_to_inky() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Pinky);
    // Now Inky's counter is active - eat dots and check Inky exits
    for _ in 0..30 {
        ctrl.on_dot_eaten();
    }
    check!(ctrl.should_exit(GhostType::Inky));
}

#[test]
fn counter_advances_inky_to_clyde() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Pinky);
    ctrl.on_ghost_exit(GhostType::Inky);
    // Now Clyde's counter is active
    for _ in 0..60 {
        ctrl.on_dot_eaten();
    }
    check!(ctrl.should_exit(GhostType::Clyde));
}

#[test]
fn counter_advances_clyde_to_none() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Pinky);
    ctrl.on_ghost_exit(GhostType::Inky);
    ctrl.on_ghost_exit(GhostType::Clyde);
    // All ghosts out -- further dots don't change exit conditions
    ctrl.on_dot_eaten();
    // No ghost type should newly qualify for exit beyond what's already true
    check!(ctrl.should_exit(GhostType::Blinky)); // always exits
    check!(ctrl.should_exit(GhostType::Pinky)); // dot limit is 0
    // Inky and Clyde: active_counter is None, so personal counter checks return false.
    // They already exited, so this is consistent -- no ghost is "waiting" to exit.
    check!(!ctrl.should_exit(GhostType::Inky));
    check!(!ctrl.should_exit(GhostType::Clyde));
}
#[test]
fn global_counter_pinky_exits_at_7() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_player_death();

    for _ in 0..6 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Pinky));

    ctrl.on_dot_eaten(); // 7th dot
    check!(ctrl.should_exit(GhostType::Pinky));
}

#[test]
fn global_counter_inky_exits_at_17() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_player_death();

    for _ in 0..16 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Inky));

    ctrl.on_dot_eaten(); // 17th dot
    check!(ctrl.should_exit(GhostType::Inky));
}

#[test]
fn global_counter_clyde_exits_at_32() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_player_death();

    for _ in 0..31 {
        ctrl.on_dot_eaten();
    }
    check!(!ctrl.should_exit(GhostType::Clyde));

    ctrl.on_dot_eaten(); // 32nd dot
    check!(ctrl.should_exit(GhostType::Clyde));
}
#[test]
fn no_dot_timer_forces_out_active_ghost() {
    let mut ctrl = GhostHouseController::new(1);
    // Level 1-4: timer limit is 4 * 60 = 240 ticks
    for _ in 0..239 {
        check!(ctrl.tick().is_none());
    }
    // On tick 240, timer hits the limit -> force out active counter
    let result = ctrl.tick();
    check!(result == Some(0)); // Pinky's index
}

#[test]
fn no_dot_timer_resets_on_dot_eaten() {
    let mut ctrl = GhostHouseController::new(1);
    // Tick 239 times (almost at limit)
    for _ in 0..239 {
        ctrl.tick();
    }
    // Eat a dot - resets the timer
    ctrl.on_dot_eaten();
    // Now need another 240 ticks
    for _ in 0..239 {
        check!(ctrl.tick().is_none());
    }
    let result = ctrl.tick();
    check!(result == Some(0));
}

#[test]
fn no_dot_timer_level_5_uses_3_seconds() {
    let mut ctrl = GhostHouseController::new(5);
    // Level 5+: timer limit is 3 * 60 = 180 ticks
    for _ in 0..179 {
        check!(ctrl.tick().is_none());
    }
    let result = ctrl.tick();
    check!(result == Some(0));
}
#[test]
fn blinky_exit_doesnt_change_counter() {
    let mut ctrl = GhostHouseController::new(1);
    ctrl.on_ghost_exit(GhostType::Blinky);
    // Pinky should still be able to exit (active_counter unchanged)
    check!(ctrl.should_exit(GhostType::Pinky));
}
