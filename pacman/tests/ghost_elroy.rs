use assert2::check;
use pacman::systems::ghost::elroy::{ElroyStage, elroy_speed, elroy_thresholds};
use rstest::rstest;

#[rstest]
#[case(1, (20, 10))]
#[case(2, (30, 15))]
#[case(3, (40, 20))]
#[case(4, (40, 20))]
#[case(5, (40, 20))]
#[case(6, (40, 20))]
#[case(7, (50, 25))]
#[case(8, (50, 25))]
#[case(9, (60, 30))]
#[case(10, (60, 30))]
#[case(11, (60, 30))]
#[case(12, (80, 40))]
#[case(14, (80, 40))]
#[case(15, (100, 50))]
#[case(17, (100, 50))]
#[case(18, (120, 60))]
#[case(255, (120, 60))]
fn thresholds_by_level(#[case] level: u8, #[case] expected: (u32, u32)) {
    check!(elroy_thresholds(level) == expected);
}

#[test]
fn speed_none_any_level_is_1() {
    check!(elroy_speed(ElroyStage::None, 1) == 1.0);
    check!(elroy_speed(ElroyStage::None, 5) == 1.0);
    check!(elroy_speed(ElroyStage::None, 20) == 1.0);
}

#[test]
fn speed_stage1_level_1() {
    check!((elroy_speed(ElroyStage::Stage1, 1) - 0.80).abs() < 0.001);
}

#[test]
fn speed_stage1_level_2_to_4() {
    check!((elroy_speed(ElroyStage::Stage1, 2) - 0.90).abs() < 0.001);
    check!((elroy_speed(ElroyStage::Stage1, 4) - 0.90).abs() < 0.001);
}

#[test]
fn speed_stage1_level_5_plus() {
    check!((elroy_speed(ElroyStage::Stage1, 5) - 1.00).abs() < 0.001);
    check!((elroy_speed(ElroyStage::Stage1, 20) - 1.00).abs() < 0.001);
}

#[test]
fn speed_stage2_level_1() {
    check!((elroy_speed(ElroyStage::Stage2, 1) - 0.85).abs() < 0.001);
}

#[test]
fn speed_stage2_level_2_to_4() {
    check!((elroy_speed(ElroyStage::Stage2, 2) - 0.95).abs() < 0.001);
    check!((elroy_speed(ElroyStage::Stage2, 4) - 0.95).abs() < 0.001);
}

#[test]
fn speed_stage2_level_5_plus_exceeds_1() {
    let speed = elroy_speed(ElroyStage::Stage2, 5);
    check!((speed - 1.05).abs() < 0.001);
    check!(speed > 1.0);
}
