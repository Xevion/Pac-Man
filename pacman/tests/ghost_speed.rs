use assert2::check;
use pacman::systems::ghost::movement::GhostSpeedConfig;
#[test]
fn level_1_normal_speed() {
    let config = GhostSpeedConfig::for_level(1);
    check!((config.normal - 0.75).abs() < 0.001);
}

#[test]
fn level_1_tunnel_speed() {
    let config = GhostSpeedConfig::for_level(1);
    check!((config.tunnel - 0.40).abs() < 0.001);
}

#[test]
fn level_1_frightened_speed() {
    let config = GhostSpeedConfig::for_level(1);
    check!((config.frightened - 0.50).abs() < 0.001);
}
#[test]
fn level_2_speeds() {
    let config = GhostSpeedConfig::for_level(2);
    check!((config.normal - 0.85).abs() < 0.001);
    check!((config.tunnel - 0.45).abs() < 0.001);
    check!((config.frightened - 0.55).abs() < 0.001);
}

#[test]
fn level_4_same_as_level_2() {
    let config = GhostSpeedConfig::for_level(4);
    check!((config.normal - 0.85).abs() < 0.001);
    check!((config.tunnel - 0.45).abs() < 0.001);
    check!((config.frightened - 0.55).abs() < 0.001);
}
#[test]
fn level_5_speeds() {
    let config = GhostSpeedConfig::for_level(5);
    check!((config.normal - 0.95).abs() < 0.001);
    check!((config.tunnel - 0.50).abs() < 0.001);
    check!((config.frightened - 0.60).abs() < 0.001);
}

#[test]
fn level_20_same_as_level_5() {
    let config = GhostSpeedConfig::for_level(20);
    check!((config.normal - 0.95).abs() < 0.001);
    check!((config.tunnel - 0.50).abs() < 0.001);
    check!((config.frightened - 0.60).abs() < 0.001);
}
#[test]
fn level_21_no_frightened_speed() {
    let config = GhostSpeedConfig::for_level(21);
    check!((config.frightened - 0.0).abs() < 0.001);
}

#[test]
fn level_21_normal_and_tunnel_same_as_5() {
    let config = GhostSpeedConfig::for_level(21);
    check!((config.normal - 0.95).abs() < 0.001);
    check!((config.tunnel - 0.50).abs() < 0.001);
}

#[test]
fn level_255_same_as_21() {
    let config = GhostSpeedConfig::for_level(255);
    check!((config.normal - 0.95).abs() < 0.001);
    check!((config.tunnel - 0.50).abs() < 0.001);
    check!((config.frightened - 0.0).abs() < 0.001);
}
#[test]
fn boundary_level_4_to_5() {
    let config4 = GhostSpeedConfig::for_level(4);
    let config5 = GhostSpeedConfig::for_level(5);
    // Level 4 (bracket 2-4) has different speeds from level 5 (bracket 5-20)
    check!(config4.normal != config5.normal);
}

#[test]
fn boundary_level_20_to_21() {
    let config20 = GhostSpeedConfig::for_level(20);
    let config21 = GhostSpeedConfig::for_level(21);
    // Normal speed is the same, but frightened differs
    check!((config20.normal - config21.normal).abs() < 0.001);
    check!(config20.frightened != config21.frightened);
}
