use bevy_ecs::{entity::Entity, query::With, world::World};
use pacman::error::{GameError, GameResult};
use pacman::game::Game;
use pacman::scenes::{Scene, SceneManager, SceneOwned};
use pacman::systems::ghost::GhostType;
use pacman::systems::state::{GameStage, Session};
use speculoos::prelude::*;

mod common;

use common::setup_sdl;

fn scene_owned_count(world: &mut World) -> usize {
    world.query_filtered::<Entity, With<SceneOwned>>().iter(world).count()
}

#[test]
fn test_game_30_seconds_60fps() -> GameResult<()> {
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(GameError::Sdl)?;
    let ttf_context = sdl2::ttf::init().map_err(GameError::Sdl)?;
    let event_pump = _sdl_context
        .event_pump()
        .map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;
    // Boot lands in the Title; enter gameplay so the loop exercises the simulation.
    game.world.resource_mut::<SceneManager>().request(Scene::Gameplay);

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
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(GameError::Sdl)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?;
    let event_pump = _sdl_context
        .event_pump()
        .map_err(|e| pacman::error::GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;
    // Boot lands in the Title; enter gameplay so the loop exercises the simulation.
    game.world.resource_mut::<SceneManager>().request(Scene::Gameplay);

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

    assert_that(&total_time).is_greater_than_or_equal_to(target_time);
    Ok(())
}

/// The gameplay lifecycle (`spawn_gameplay` -> `despawn_gameplay` -> `spawn_gameplay`)
/// must leave no leaked entities and no dangling `Entity` ids behind.
#[test]
fn gameplay_teardown_and_respawn_leave_no_leaks() -> GameResult<()> {
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(GameError::Sdl)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?;
    let event_pump = _sdl_context.event_pump().map_err(|e| GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;

    // Boot lands in the Title (no gameplay entities); populate one scene to tear down.
    pacman::game::spawning::spawn_gameplay(&mut game.world, 1)?;
    let initial = scene_owned_count(&mut game.world);
    assert_that(&initial).is_greater_than(0);

    // Seed the dangling-`Entity` hazard teardown must clear: a `GhostEatenPause` stage
    // referencing a real ghost that is about to be despawned.
    let ghost_entity = {
        let mut query = game.world.query_filtered::<Entity, With<GhostType>>();
        query.iter(&game.world).next().expect("boot spawns ghosts")
    };
    game.world.resource_mut::<Session>().set_stage(GameStage::GhostEatenPause {
        remaining_ticks: 30,
        ghost_entity,
        ghost_type: GhostType::Blinky,
        node: 0,
    });

    // Tearing it down removes every scene entity and drops the dangling stage reference.
    pacman::game::spawning::despawn_gameplay(&mut game.world);
    assert_that(&scene_owned_count(&mut game.world)).is_equal_to(0);
    let stage_holds_entity = matches!(game.world.resource::<Session>().stage(), GameStage::GhostEatenPause { .. });
    assert_that(&stage_holds_entity).is_false();

    // Respawning restores the same population; the full cycle leaks nothing.
    pacman::game::spawning::spawn_gameplay(&mut game.world, 1)?;
    assert_that(&scene_owned_count(&mut game.world)).is_equal_to(initial);
    Ok(())
}

/// Boot lands in the Title scene with no gameplay entities; requesting Gameplay and
/// ticking once must route through `apply_pending_scene` and populate the scene.
#[test]
fn scene_router_boots_in_title_then_enters_gameplay() -> GameResult<()> {
    let (canvas, texture_creator, _sdl_context) = setup_sdl().map_err(GameError::Sdl)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?;
    let event_pump = _sdl_context.event_pump().map_err(|e| GameError::Sdl(e.to_string()))?;

    let mut game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;

    // Boot scene is the Title, which owns no entities.
    assert_that(&game.world.resource::<SceneManager>().active()).is_equal_to(Scene::Title);
    assert_that(&scene_owned_count(&mut game.world)).is_equal_to(0);

    // Queue Gameplay; the router applies it at the top of the next tick.
    game.world.resource_mut::<SceneManager>().request(Scene::Gameplay);
    game.tick(1.0 / 60.0);

    assert_that(&game.world.resource::<SceneManager>().active()).is_equal_to(Scene::Gameplay);
    assert_that(&scene_owned_count(&mut game.world)).is_greater_than(0);
    Ok(())
}
