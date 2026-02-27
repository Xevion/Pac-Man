use pacman::systems::ghost::GhostSpeedConfig;
use speculoos::prelude::*;
#[test]
fn level_1_normal_speed() {
    let config = GhostSpeedConfig::for_level(1);
    assert_that(&config.normal).is_close_to(0.75, 0.001);
}

#[test]
fn level_1_tunnel_speed() {
    let config = GhostSpeedConfig::for_level(1);
    assert_that(&config.tunnel).is_close_to(0.40, 0.001);
}

#[test]
fn level_1_frightened_speed() {
    let config = GhostSpeedConfig::for_level(1);
    assert_that(&config.frightened).is_close_to(0.50, 0.001);
}
#[test]
fn level_2_speeds() {
    let config = GhostSpeedConfig::for_level(2);
    assert_that(&config.normal).is_close_to(0.85, 0.001);
    assert_that(&config.tunnel).is_close_to(0.45, 0.001);
    assert_that(&config.frightened).is_close_to(0.55, 0.001);
}

#[test]
fn level_4_same_as_level_2() {
    let config = GhostSpeedConfig::for_level(4);
    assert_that(&config.normal).is_close_to(0.85, 0.001);
    assert_that(&config.tunnel).is_close_to(0.45, 0.001);
    assert_that(&config.frightened).is_close_to(0.55, 0.001);
}
#[test]
fn level_5_speeds() {
    let config = GhostSpeedConfig::for_level(5);
    assert_that(&config.normal).is_close_to(0.95, 0.001);
    assert_that(&config.tunnel).is_close_to(0.50, 0.001);
    assert_that(&config.frightened).is_close_to(0.60, 0.001);
}

#[test]
fn level_20_same_as_level_5() {
    let config = GhostSpeedConfig::for_level(20);
    assert_that(&config.normal).is_close_to(0.95, 0.001);
    assert_that(&config.tunnel).is_close_to(0.50, 0.001);
    assert_that(&config.frightened).is_close_to(0.60, 0.001);
}
#[test]
fn level_21_no_frightened_speed() {
    let config = GhostSpeedConfig::for_level(21);
    assert_that(&config.frightened).is_close_to(0.0, 0.001);
}

#[test]
fn level_21_normal_and_tunnel_same_as_5() {
    let config = GhostSpeedConfig::for_level(21);
    assert_that(&config.normal).is_close_to(0.95, 0.001);
    assert_that(&config.tunnel).is_close_to(0.50, 0.001);
}

#[test]
fn level_255_same_as_21() {
    let config = GhostSpeedConfig::for_level(255);
    assert_that(&config.normal).is_close_to(0.95, 0.001);
    assert_that(&config.tunnel).is_close_to(0.50, 0.001);
    assert_that(&config.frightened).is_close_to(0.0, 0.001);
}
#[test]
fn boundary_level_4_to_5() {
    let config4 = GhostSpeedConfig::for_level(4);
    let config5 = GhostSpeedConfig::for_level(5);
    // Level 4 (bracket 2-4) has different speeds from level 5 (bracket 5-20)
    assert_that(&config4.normal).is_not_equal_to(config5.normal);
}

#[test]
fn boundary_level_20_to_21() {
    let config20 = GhostSpeedConfig::for_level(20);
    let config21 = GhostSpeedConfig::for_level(21);
    // Normal speed is the same, but frightened differs
    assert_that(&config20.normal).is_close_to(config21.normal, 0.001);
    assert_that(&config20.frightened).is_not_equal_to(config21.frightened);
}
