use pacman::texture::blinking::BlinkingTexture;
use speculoos::prelude::*;

mod common;

#[test]
fn test_blinking_texture() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 30); // 30 ticks = 0.5 seconds at 60 FPS

    assert_that(&texture.is_on()).is_true();

    texture.tick(30);
    assert_that(&texture.is_on()).is_false();

    texture.tick(30);
    assert_that(&texture.is_on()).is_true();

    texture.tick(30);
    assert_that(&texture.is_on()).is_false();
}

#[test]
fn test_blinking_texture_partial_duration() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 30); // 30 ticks

    texture.tick(37); // 37 ticks, should complete 1 interval (30 ticks) with 7 remaining
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(7);
}

#[test]
fn test_blinking_texture_zero_interval() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 0);

    assert_that(&texture.is_on()).is_true();
    assert_that(&texture.interval_ticks()).is_equal_to(0);

    // With zero interval, any positive ticks should toggle
    texture.tick(1);
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(1);

    texture.tick(1);
    assert_that(&texture.is_on()).is_true();
    assert_that(&texture.tick_timer()).is_equal_to(2);
}

#[test]
fn test_blinking_texture_multiple_cycles() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 15); // 15 ticks

    // Test multiple complete cycles
    for i in 0..10 {
        let expected_state = i % 2 == 0;
        assert_that(&texture.is_on()).is_equal_to(expected_state);
        texture.tick(15);
    }
}

#[test]
fn test_blinking_texture_accumulated_ticks() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 60); // 60 ticks = 1 second at 60 FPS

    // Test that ticks accumulate correctly
    texture.tick(18); // 18 ticks
    assert_that(&texture.tick_timer()).is_equal_to(18);
    assert_that(&texture.is_on()).is_true();

    texture.tick(18); // 36 total ticks
    assert_that(&texture.tick_timer()).is_equal_to(36);
    assert_that(&texture.is_on()).is_true();

    texture.tick(18); // 54 total ticks
    assert_that(&texture.tick_timer()).is_equal_to(54);
    assert_that(&texture.is_on()).is_true();

    texture.tick(12); // 66 total ticks, should complete 1 interval (60 ticks) with 6 remaining
    assert_that(&texture.tick_timer()).is_equal_to(6);
    assert_that(&texture.is_on()).is_false();
}

#[test]
fn test_blinking_texture_large_tick_step() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 30); // 30 ticks

    // Test with a tick step larger than the interval
    texture.tick(90); // 90 ticks, should complete 3 intervals (90/30 = 3)
    assert_that(&texture.is_on()).is_false(); // 3 toggles: true -> false -> true -> false
    assert_that(&texture.tick_timer()).is_equal_to(0); // 90 % 30 = 0

    texture.tick(60); // 60 ticks, should complete 2 intervals (60/30 = 2)
    assert_that(&texture.is_on()).is_false(); // 2 more toggles: false -> true -> false (no change)
    assert_that(&texture.tick_timer()).is_equal_to(0); // 60 % 30 = 0
}

#[test]
fn test_blinking_texture_small_steps() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 6); // 6 ticks

    // Test with small tick steps
    for _ in 0..6 {
        texture.tick(1);
    }

    // After 6 ticks, should have completed one cycle
    assert_that(&texture.tick_timer()).is_equal_to(0);
    assert_that(&texture.is_on()).is_false();
}

#[test]
fn test_blinking_texture_clone() {
    let tile = common::mock_atlas_tile(1);
    let mut texture1 = BlinkingTexture::new(tile, 30);
    let texture2 = texture1.clone();

    // Both should have the same initial state
    assert_that(&texture1.is_on()).is_equal_to(texture2.is_on());
    assert_that(&texture1.tick_timer()).is_equal_to(texture2.tick_timer());
    assert_that(&texture1.interval_ticks()).is_equal_to(texture2.interval_ticks());

    // Modifying one shouldn't affect the other
    texture1.tick(18);
    assert_that(&texture1.tick_timer()).is_not_equal_to(texture2.tick_timer());
    assert_that(&texture2.tick_timer()).is_equal_to(0);
}

#[test]
fn test_blinking_texture_edge_cases() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 60); // 60 ticks

    // Test exactly at the interval
    texture.tick(60);
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(0);

    // Test just under the interval
    texture.tick(59);
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(59);

    // Test just over the interval
    texture.tick(2);
    assert_that(&texture.is_on()).is_true();
    assert_that(&texture.tick_timer()).is_equal_to(1);
}

#[test]
fn test_blinking_texture_very_small_interval() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 1); // 1 tick

    // Test with very small interval
    texture.tick(1);
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(0);

    texture.tick(1);
    assert_that(&texture.is_on()).is_true();
    assert_that(&texture.tick_timer()).is_equal_to(0);
}

#[test]
fn test_blinking_texture_very_large_interval() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 60000); // 60000 ticks = 1000 seconds at 60 FPS

    // Test with very large interval
    texture.tick(30000);
    assert_that(&texture.is_on()).is_true();
    assert_that(&texture.tick_timer()).is_equal_to(30000);

    texture.tick(30000);
    assert_that(&texture.is_on()).is_false();
    assert_that(&texture.tick_timer()).is_equal_to(0);
}

#[test]
fn test_blinking_texture_multiple_toggles_no_op() {
    let tile = common::mock_atlas_tile(1);
    let mut texture = BlinkingTexture::new(tile, 10); // 10 ticks

    // Test that multiple toggles work correctly (key feature)
    // 2x ticks than the interval should do nothing at all, because toggling twice is a no-op
    texture.tick(20); // 20 ticks = 2 complete intervals
    assert_that(&texture.is_on()).is_true(); // Should still be true (2 toggles = no-op)
    assert_that(&texture.tick_timer()).is_equal_to(0);

    // 3x ticks should toggle once (3 toggles = 1 toggle)
    texture.tick(30); // 30 ticks = 3 complete intervals
    assert_that(&texture.is_on()).is_false(); // Should be false (3 toggles = 1 toggle)
    assert_that(&texture.tick_timer()).is_equal_to(0);

    // 4x ticks should do nothing (4 toggles = no-op)
    texture.tick(40); // 40 ticks = 4 complete intervals
    assert_that(&texture.is_on()).is_false(); // Should still be false (4 toggles = no-op)
    assert_that(&texture.tick_timer()).is_equal_to(0);
}
