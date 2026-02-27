use pacman::systems::ghost::elroy::{elroy_speed, elroy_thresholds, ElroyStage};
use speculoos::prelude::*;

#[test]
fn thresholds_by_level() {
    let cases: &[(u8, (u32, u32))] = &[
        (1, (20, 10)),
        (2, (30, 15)),
        (3, (40, 20)),
        (4, (40, 20)),
        (5, (40, 20)),
        (6, (40, 20)),
        (7, (50, 25)),
        (8, (50, 25)),
        (9, (60, 30)),
        (10, (60, 30)),
        (11, (60, 30)),
        (12, (80, 40)),
        (14, (80, 40)),
        (15, (100, 50)),
        (17, (100, 50)),
        (18, (120, 60)),
        (255, (120, 60)),
    ];
    for &(level, expected) in cases {
        assert_that(&elroy_thresholds(level)).is_equal_to(expected);
    }
}
#[test]
fn speed_none_any_level_is_1() {
    assert_that(&elroy_speed(ElroyStage::None, 1)).is_equal_to(1.0);
    assert_that(&elroy_speed(ElroyStage::None, 5)).is_equal_to(1.0);
    assert_that(&elroy_speed(ElroyStage::None, 20)).is_equal_to(1.0);
}

#[test]
fn speed_stage1_level_1() {
    assert_that(&elroy_speed(ElroyStage::Stage1, 1)).is_close_to(0.80, 0.001);
}

#[test]
fn speed_stage1_level_2_to_4() {
    assert_that(&elroy_speed(ElroyStage::Stage1, 2)).is_close_to(0.90, 0.001);
    assert_that(&elroy_speed(ElroyStage::Stage1, 4)).is_close_to(0.90, 0.001);
}

#[test]
fn speed_stage1_level_5_plus() {
    assert_that(&elroy_speed(ElroyStage::Stage1, 5)).is_close_to(1.00, 0.001);
    assert_that(&elroy_speed(ElroyStage::Stage1, 20)).is_close_to(1.00, 0.001);
}

#[test]
fn speed_stage2_level_1() {
    assert_that(&elroy_speed(ElroyStage::Stage2, 1)).is_close_to(0.85, 0.001);
}

#[test]
fn speed_stage2_level_2_to_4() {
    assert_that(&elroy_speed(ElroyStage::Stage2, 2)).is_close_to(0.95, 0.001);
    assert_that(&elroy_speed(ElroyStage::Stage2, 4)).is_close_to(0.95, 0.001);
}

#[test]
fn speed_stage2_level_5_plus_exceeds_1() {
    let speed = elroy_speed(ElroyStage::Stage2, 5);
    assert_that(&speed).is_close_to(1.05, 0.001);
    assert_that(&speed).is_greater_than(1.0);
}
