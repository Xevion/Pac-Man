//! Core game module -- owns the ECS world and schedule, delegates initialization
//! to focused submodules.

mod animations;
mod init;
mod schedule;
pub mod spawning;

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

use tracing::{debug, info, warn};

use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::constants;
use crate::error::{GameError, GameResult};
use crate::map::builder::Map;
use crate::map::render::MapRenderer;
use crate::platform;
use crate::systems;
use crate::systems::common::{DeltaTime, GlobalState};
use crate::systems::profiling::Timing;

/// Core game state manager built on the Bevy ECS architecture.
///
/// Orchestrates all game systems through a centralized `World` containing entities,
/// components, and resources, while a `Schedule` defines system execution order.
/// Handles initialization of graphics resources, entity spawning, and per-frame
/// game logic coordination. SDL2 resources are stored as `NonSend` to respect
/// thread safety requirements while integrating with the ECS.
pub struct Game {
    pub world: World,
    pub schedule: Schedule,
}

impl Game {
    /// Initializes the complete game state including ECS world, graphics, and entity spawning.
    ///
    /// # Errors
    ///
    /// Returns `GameError` for SDL2 failures, asset loading problems, atlas parsing
    /// errors, or entity initialization issues.
    pub fn new(
        mut canvas: Canvas<Window>,
        ttf_context: sdl2::ttf::Sdl2TtfContext,
        texture_creator: TextureCreator<WindowContext>,
        mut event_pump: EventPump,
    ) -> GameResult<Game> {
        info!("Starting game initialization");

        debug!("Disabling unnecessary SDL events");
        init::disable_sdl_events(&mut event_pump);

        debug!("Setting up textures and fonts");
        let (backbuffer, mut map_texture, ttf_atlas) =
            init::setup_textures_and_fonts(&mut canvas, &texture_creator, ttf_context)?;
        platform::yield_to_browser();

        debug!("Initializing audio subsystem");
        let audio = crate::audio::Audio::new();
        platform::yield_to_browser();

        debug!("Loading sprite atlas and map tiles");
        let (mut atlas, map_tiles) = init::load_atlas_and_map_tiles(&texture_creator, &ATLAS_FRAMES)?;
        platform::yield_to_browser();

        debug!("Rendering static map to texture cache");
        canvas
            .with_texture_canvas(&mut map_texture, |map_canvas| {
                MapRenderer::render_map(map_canvas, &mut atlas, &map_tiles);
            })
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        platform::yield_to_browser();

        debug!("Building navigation graph from map layout");
        let map = Map::new(constants::RAW_BOARD)?;

        debug!("Initializing ghost AI special nodes");
        let red_zones = crate::systems::ghost::RedZoneNodes::from_map(&map);
        let tunnel_nodes = crate::systems::ghost::TunnelNodes::from_map(&map);

        debug!("Creating death animation sequence");
        let death_animation = animations::create_death_animation(&atlas)?;

        debug!("Initializing ECS world and system schedule");
        let mut world = World::default();
        let mut schedule = Schedule::default();

        debug!("Setting up ECS event registry and observers");
        init::setup_ecs(&mut world);

        world.add_observer(systems::item::spawn_fruit_observer);

        debug!("Inserting resources into ECS world");
        init::insert_resources(
            &mut world,
            map,
            init::InitResources {
                audio,
                atlas,
                event_pump,
                canvas,
                backbuffer,
                map_texture,
                ttf_atlas,
                death_animation,
                red_zones,
                tunnel_nodes,
            },
        )?;

        debug!("Configuring system execution schedule");
        schedule::configure_schedule(&mut schedule);

        info!("Entering initial scene");
        let scenes = crate::scenes::SceneManager::new(crate::scenes::Scene::Title);
        scenes.enter_initial(&mut world)?;
        world.insert_resource(scenes);

        info!("Game initialization completed successfully");
        Ok(Game { world, schedule })
    }

    /// Starts the game after the first user interaction (Emscripten only).
    ///
    /// Unlocks audio within the user gesture (browser autoplay policy) and queues
    /// the Gameplay scene, leaving the Title. Called from JavaScript when the user
    /// clicks or presses a key.
    #[cfg(target_os = "emscripten")]
    pub fn start(&mut self) {
        use crate::scenes::{Scene, SceneManager};
        use crate::systems::audio::AudioResource;

        // Unlock audio now that the user has interacted; autoplay policy requires
        // this to happen from within the gesture.
        if let Some(mut audio) = self.world.get_non_send_resource_mut::<AudioResource>() {
            audio.0.unlock();
        }

        // Queue the Gameplay scene; the router applies it at the top of the next frame.
        if let Some(mut scenes) = self.world.get_resource_mut::<SceneManager>() {
            if scenes.active() == Scene::Title {
                tracing::info!("User interaction detected, starting game");
                scenes.request(Scene::Gameplay);
            }
        }
    }

    /// Executes one frame of game logic by running all scheduled ECS systems.
    ///
    /// # Arguments
    ///
    /// * `dt` - Frame delta time in seconds for time-based animations and movement
    ///
    /// # Returns
    ///
    /// `true` if the game should terminate (exit command received), `false` to continue
    pub fn tick(&mut self, dt: f32) -> bool {
        let _tick = tracing::debug_span!("tick").entered();
        self.world.insert_resource(DeltaTime { seconds: dt, ticks: 1 });

        let start = std::time::Instant::now();
        {
            let _run = tracing::debug_span!("schedule_run").entered();
            self.schedule.run(&mut self.world);
        }
        let total_duration = start.elapsed();
        crate::tracy_plot!("frame.schedule_ms", total_duration.as_secs_f64() * 1000.0);

        if let (Some(timings), Some(timing)) = (
            self.world.get_resource::<systems::profiling::SystemTimings>(),
            self.world.get_resource::<Timing>(),
        ) {
            let new_tick = timing.increment_tick();
            timings.add_total_timing(total_duration, new_tick);

            let frame_budget_ms = (dt * 1000.0 * 1.2) as u128;

            if total_duration.as_millis() > frame_budget_ms {
                let slowest_systems = timings.get_slowest_systems();
                let systems_context = if slowest_systems.is_empty() {
                    "No specific systems identified".to_string()
                } else {
                    slowest_systems
                        .iter()
                        .map(|(id, duration)| format!("{} ({:.2?})", id, duration))
                        .collect::<Vec<String>>()
                        .join(", ")
                };

                warn!(
                    total = format!("{:.3?}", total_duration),
                    tick = new_tick,
                    systems = systems_context,
                    budget = format!("{:.1}ms", frame_budget_ms),
                    "Frame took longer than expected"
                );
            }
        }

        let state = self
            .world
            .get_resource::<GlobalState>()
            .expect("GlobalState could not be acquired");

        state.exit
    }
}
