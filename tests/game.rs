use pacman::error::GameResult;
use pacman::game::Game;
use sdl2;

mod common;

use common::setup_sdl;

/// Test that runs the game for 30 seconds at 60 FPS without sleep
#[test]
fn test_game_30_seconds_60fps() -> GameResult<()> {
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(|e| pacman::error::GameError::Sdl(e))?;
    let ttf_context = sdl2::ttf::init().map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;
    let event_pump = _sdl_context
        .event_pump()
        .map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;

    // Run for 30 seconds at 60 FPS = 1800 frames
    let frame_time = 1.0 / 60.0;
    let total_frames = 1800;
    let mut frame_count = 0;

    for _ in 0..total_frames {
        let should_exit = game.tick(frame_time);

        if should_exit {
            break;
        }

        frame_count += 1;
    }

    assert_eq!(
        frame_count, total_frames,
        "Should have processed exactly {} frames",
        total_frames
    );
    Ok(())
}

/// Test that runs the game for 30 seconds with variable frame timing
#[test]
fn test_game_30_seconds_variable_timing() -> GameResult<()> {
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(|e| pacman::error::GameError::Sdl(e))?;
    let ttf_context = sdl2::ttf::init().map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;
    let event_pump = _sdl_context
        .event_pump()
        .map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;

    // Simulate 30 seconds with variable frame timing
    let mut total_time = 0.0;
    let target_time = 30.0;
    let mut frame_count = 0;

    while total_time < target_time {
        // Alternate between different frame rates to simulate real gameplay
        let frame_time = match frame_count % 4 {
            0 => 1.0 / 60.0,  // 60 FPS
            1 => 1.0 / 30.0,  // 30 FPS (lag spike)
            2 => 1.0 / 120.0, // 120 FPS (very fast)
            _ => 1.0 / 60.0,  // 60 FPS
        };

        let should_exit = game.tick(frame_time);

        if should_exit {
            break;
        }

        total_time += frame_time;
        frame_count += 1;
    }

    assert!(
        total_time >= target_time,
        "Should have run for at least {} seconds, but ran for {}s",
        target_time,
        total_time
    );
    Ok(())
}
