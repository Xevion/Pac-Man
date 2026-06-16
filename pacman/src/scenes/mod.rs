//! Scene layer: the top-altitude identity of what the game is currently showing.
//!
//! A [`Scene`] is the lightweight identity tag (a `Copy` enum used in components,
//! resources, and run-conditions). Its behavior lives in a [`SceneHandler`]
//! implementation per scene, mapped from the tag by [`Scene::handler`]. Every
//! entity a scene spawns is tagged [`SceneOwned`] so leaving the scene despawns
//! exactly its own entities. [`SceneManager`] owns the active scene plus any queued
//! transition and applies it between frames.
//!
//! Adding a scene is: define its handler type, add an enum variant, and wire the
//! one exhaustive `handler` arm -- the compiler forces all three.

use bevy_ecs::component::Component;
use bevy_ecs::event::EventReader;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::world::World;

use crate::error::GameResult;
use crate::events::{GameCommand, GameEvent};
use crate::systems::state::PauseState;

mod attract;
mod gameplay;
mod title;

pub use title::title_input_system;

/// The behavior of a scene: what happens when it becomes active and when it is
/// left. Implemented once per scene by a zero-sized handler type.
///
/// Both hooks run while the [`SceneManager`] is mid-transition (temporarily out of
/// the world), so a hook must not read the `SceneManager` resource -- it is inside
/// its own transition.
pub trait SceneHandler {
    /// Bring the scene up: spawn its entities, set up its state.
    fn on_enter(&self, world: &mut World) -> GameResult<()>;
    /// Tear the scene down: despawn its entities, clear any state it owned.
    fn on_exit(&self, world: &mut World);
}

/// The top-level screen the game is currently presenting -- the lightweight tag
/// stored in components and resources. Behavior lives in [`Scene::handler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    /// The press-to-start screen shown at boot.
    Title,
    /// The playable maze: player, ghosts, and collectibles.
    Gameplay,
    /// A self-playing demo: the gameplay maze under AI control.
    Attract,
}

impl Scene {
    /// Maps a scene tag to its behavior. Exhaustive: a new variant won't compile
    /// until it has a handler.
    fn handler(self) -> &'static dyn SceneHandler {
        match self {
            Scene::Title => &title::TitleScene,
            Scene::Gameplay => &gameplay::GameplayScene,
            Scene::Attract => &attract::AttractScene,
        }
    }
}

/// Tags an entity as belonging to a particular [`Scene`]. When that scene is torn
/// down, every entity carrying this component is despawned together, so no scene
/// entity outlives the scene that owns it.
#[derive(Component, Debug, Clone, Copy)]
pub struct SceneOwned(pub Scene);

/// Owns the active scene and any pending transition to apply between frames.
///
/// Transitions are deferred rather than applied the instant they're requested
/// because [`SceneHandler::on_enter`]/[`SceneHandler::on_exit`] need exclusive
/// `&mut World` access -- they spawn and despawn whole entity populations.
/// [`apply_pending_scene`] runs at the top of the frame, before any gameplay
/// system, so a freshly entered scene's entities exist for the rest of that frame.
#[derive(Resource, Debug)]
pub struct SceneManager {
    active: Scene,
    pending: Option<Scene>,
    /// When set, re-enter `active` in place (on_exit then on_enter) at the next
    /// apply -- rebuilding its entities without changing scene. Drives the debug
    /// `ResetLevel` command.
    reload: bool,
}

impl SceneManager {
    /// Creates a manager already in `active` with nothing pending. The initial
    /// scene's [`SceneHandler::on_enter`] is run separately via [`Self::enter_initial`].
    pub fn new(active: Scene) -> Self {
        Self {
            active,
            pending: None,
            reload: false,
        }
    }

    /// The scene currently being presented.
    pub fn active(&self) -> Scene {
        self.active
    }

    /// Queues a transition to `scene`, applied at the top of the next frame by
    /// [`apply_pending_scene`]. A later request in the same frame overrides an
    /// earlier one; requesting the already-active scene collapses to a no-op.
    pub fn request(&mut self, scene: Scene) {
        self.pending = Some(scene);
    }

    /// Queues an in-place rebuild of the active scene (despawn + respawn) for the
    /// next apply. Used by the debug `ResetLevel` command.
    pub fn reload(&mut self) {
        self.reload = true;
    }

    /// Runs the active scene's enter hook once at boot. Call before inserting the
    /// manager into the world, so the hook has unobstructed `&mut World` access and
    /// there is no prior scene to exit.
    pub fn enter_initial(&self, world: &mut World) -> GameResult<()> {
        self.active.handler().on_enter(world)
    }

    /// Applies a queued transition: exit the active scene, enter the new one, make
    /// it active. Called from [`apply_pending_scene`] with the manager removed from
    /// the world, so it holds `&mut self` and `&mut World` at once.
    fn apply_pending(&mut self, world: &mut World) {
        if let Some(next) = self.pending.take() {
            if next != self.active {
                // A scene switch rebuilds entities wholesale, so any queued reload is moot.
                self.reload = false;
                self.active.handler().on_exit(world);
                if let Err(e) = next.handler().on_enter(world) {
                    // Entering a scene only fails on missing assets, which would already
                    // have failed at boot, so this is effectively unreachable post-startup.
                    // Log rather than panic so a transient failure can't take down the loop.
                    tracing::error!("entering scene {next:?} failed: {e}");
                }
                self.active = next;
                return;
            }
        }

        // No scene change requested: honor a queued in-place reload by tearing the
        // active scene down and bringing it straight back up.
        if std::mem::take(&mut self.reload) {
            self.active.handler().on_exit(world);
            if let Err(e) = self.active.handler().on_enter(world) {
                tracing::error!("reloading scene {:?} failed: {e}", self.active);
            }
        }
    }
}

/// Exclusive system at the top of the schedule: apply any pending scene transition.
///
/// The manager is pulled out of the world for the duration so the transition logic
/// can take `&mut self` alongside `&mut World`; it is reinserted immediately after.
pub fn apply_pending_scene(world: &mut World) {
    let mut scenes = world
        .remove_resource::<SceneManager>()
        .expect("SceneManager resource is present for the whole run");
    scenes.apply_pending(world);
    world.insert_resource(scenes);
}

/// Run-condition: true only while `scene` is active. Gates per-scene systems.
pub fn in_scene(scene: Scene) -> impl Fn(Res<SceneManager>) -> bool + Clone {
    move |scenes: Res<SceneManager>| scenes.active() == scene
}

/// Run-condition for the simulation sets (Update/Respond/Animation): the active
/// scene is a live gameplay simulation (player-driven Gameplay or AI-driven Attract)
/// and the game isn't paused.
pub fn sim_active(scenes: Res<SceneManager>, pause: Res<PauseState>) -> bool {
    matches!(scenes.active(), Scene::Gameplay | Scene::Attract) && !pause.active()
}

/// Handles the debug `ResetLevel` command by queuing an in-place reload of the
/// active scene; the rebuild itself happens next frame in [`apply_pending_scene`].
pub fn handle_reset_command(mut events: EventReader<GameEvent>, mut scenes: ResMut<SceneManager>) {
    if events
        .read()
        .any(|e| matches!(e, GameEvent::Command(GameCommand::ResetLevel)))
    {
        scenes.reload();
    }
}
