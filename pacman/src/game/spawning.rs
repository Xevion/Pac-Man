//! Entity spawning for player, ghosts, and collectible items.

use tracing::{info, trace};

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

use crate::constants;
use crate::constants::MapTile;
use crate::error::GameResult;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::scenes::{Scene, SceneOwned};
use crate::systems::animation::Blinking;
use crate::systems::collision::Collider;
use crate::systems::common::{EntityType, Frozen, Ghost, Item, Pacman};
use crate::systems::ghost::{GhostAnimations, GhostHouseController, GhostModeController, GhostType};
use crate::systems::item::PelletCount;
use crate::systems::movement::{NodeId, Position, Velocity};
use crate::systems::render::{Renderable, Visibility};
use crate::systems::state::{GameStage, Session};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{GameSprite, GhostSprite, MazeSprite};

/// Spawns a fresh gameplay scene for `level`: resets the per-level controllers and
/// pellet progress, then spawns the player, ghosts, and collectibles -- each tagged
/// [`SceneOwned`] so the scene can be torn down as a unit. Score and lives belong to
/// the session and are deliberately left untouched here.
pub fn spawn_gameplay(world: &mut World, level: u8) -> GameResult<()> {
    configure_level(world, level);
    spawn_player(world)?;
    spawn_ghosts(world)?;
    spawn_items(world)?;
    Ok(())
}

/// Tears down the gameplay scene without leaking entities or dangling `Entity` ids.
///
/// Meant to run between frames, once the schedule has drained its command queue; the
/// leading `world.flush()` is then defensive cover for at most one queued layer.
/// Order is load-bearing: queued commands are flushed first so any pending observer
/// triggers (e.g. `CollisionTrigger`) resolve against still-live entities; then the
/// `Entity` id held in gameplay state (`Session::stage`) is dropped so nothing points at
/// a despawned entity; finally every `SceneOwned` entity is despawned.
pub fn despawn_gameplay(world: &mut World) {
    world.flush();

    if let Some(mut session) = world.get_resource_mut::<Session>() {
        if matches!(session.stage(), GameStage::GhostEatenPause { .. }) {
            session.set_stage(GameStage::initial());
        }
    }

    let doomed: Vec<Entity> = world
        .query::<(Entity, &SceneOwned)>()
        .iter(world)
        .filter_map(|(entity, owned)| (owned.0 == Scene::Gameplay).then_some(entity))
        .collect();
    for entity in doomed {
        world.despawn(entity);
    }

    debug_assert!(
        world
            .query::<&SceneOwned>()
            .iter(world)
            .all(|owned| owned.0 != Scene::Gameplay),
        "despawn_gameplay left Gameplay-owned entities alive"
    );
}

/// Resets the level-derived state to `level` through the single update path: the
/// session's level and pellet progress, plus both ghost controllers.
fn configure_level(world: &mut World, level: u8) {
    {
        let mut session = world.resource_mut::<Session>();
        session.level = level;
        session.pellets = PelletCount::default();
        session.ghost_eaten_chain = 0;
    }
    world.resource_mut::<GhostModeController>().reset(level);
    world.resource_mut::<GhostHouseController>().reset(level);
}

fn spawn_player(world: &mut World) -> GameResult<()> {
    let pacman_node = world.resource::<Map>().start_positions.pacman;
    let bundle = {
        let atlas = world.non_send_resource::<SpriteAtlas>();
        let (animation, start_sprite) = super::animations::create_player_animations(atlas)?;
        (
            Pacman,
            Position::Stopped { node: pacman_node },
            Renderable {
                sprite: start_sprite,
                layer: 0,
            },
            animation,
        )
    };
    world
        .spawn(bundle)
        .insert((Frozen, Visibility::hidden(), SceneOwned(Scene::Gameplay)));
    Ok(())
}

fn spawn_items(world: &mut World) -> GameResult<()> {
    trace!("Loading item sprites from atlas");
    let pellet_sprite = SpriteAtlas::get_tile(
        world.non_send_resource::<SpriteAtlas>(),
        &GameSprite::Maze(MazeSprite::Pellet).to_path(),
    )?;
    let energizer_sprite = SpriteAtlas::get_tile(
        world.non_send_resource::<SpriteAtlas>(),
        &GameSprite::Maze(MazeSprite::Energizer).to_path(),
    )?;

    let nodes: Vec<(NodeId, EntityType, AtlasTile, f32)> = world
        .resource::<Map>()
        .iter_nodes()
        .filter_map(|(id, tile)| match tile {
            MapTile::Pellet => Some((*id, EntityType::Pellet, pellet_sprite, constants::collider::PELLET_SIZE)),
            MapTile::PowerPellet => Some((
                *id,
                EntityType::PowerPellet,
                energizer_sprite,
                constants::collider::POWER_PELLET_SIZE,
            )),
            _ => None,
        })
        .collect();

    info!(
        pellet_count = nodes.iter().filter(|(_, t, _, _)| *t == EntityType::Pellet).count(),
        power_pellet_count = nodes.iter().filter(|(_, t, _, _)| *t == EntityType::PowerPellet).count(),
        "Spawning collectible items"
    );

    for (id, item_type, sprite, size) in nodes {
        let mut item = world.spawn((
            Item,
            Position::Stopped { node: id },
            Renderable { sprite, layer: 1 },
            item_type,
            Collider { size },
            SceneOwned(Scene::Gameplay),
        ));

        if item_type == EntityType::PowerPellet {
            item.insert((Frozen, Blinking::new(constants::ui::POWER_PELLET_BLINK_RATE)));
        }
    }
    Ok(())
}

/// Creates and spawns all four ghosts with unique AI personalities and directional animations.
///
/// # Errors
///
/// Returns `GameError::Texture` if any ghost sprite cannot be found in the atlas,
/// typically indicating missing or misnamed sprite files.
fn spawn_ghosts(world: &mut World) -> GameResult<()> {
    trace!("Spawning ghost entities with AI personalities");
    // Extract the data we need first to avoid borrow conflicts
    let ghost_start_positions = {
        let map = world.resource::<Map>();
        [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde].map(|g| (g, map.start_positions.ghost_start(g)))
    };

    for (ghost_type, start_node) in ghost_start_positions {
        // Create the ghost bundle in a separate scope to manage borrows
        let (ghost_bundle, extra_components) = {
            let animations = world.resource::<GhostAnimations>().get_normal(&ghost_type).unwrap().clone();
            let atlas = world.non_send_resource::<SpriteAtlas>();
            let sprite_path = GameSprite::Ghost(GhostSprite::Normal(ghost_type, Direction::Left, 0)).to_path();

            let ghost_state = ghost_type.initial_state();

            let bundle = (
                Ghost,
                ghost_type,
                Position::Stopped { node: start_node },
                Velocity {
                    speed: ghost_type.base_speed(),
                    direction: Direction::Left,
                },
                Renderable {
                    sprite: SpriteAtlas::get_tile(atlas, &sprite_path)?,
                    layer: 0,
                },
                animations,
                ghost_state,
            );

            // Blinky gets additional components
            let extra = if ghost_type == GhostType::Blinky {
                Some((crate::systems::ghost::BlinkyMarker, crate::systems::ghost::Elroy::default()))
            } else {
                None
            };

            (bundle, extra)
        };

        let mut entity_commands = world.spawn(ghost_bundle);
        entity_commands.insert((Frozen, Visibility::hidden(), SceneOwned(Scene::Gameplay)));

        if let Some((marker, elroy)) = extra_components {
            entity_commands.insert((marker, elroy));
        }

        let entity = entity_commands.id();
        trace!(ghost = ?ghost_type, entity = ?entity, start_node, "Spawned ghost entity");
    }

    info!("All ghost entities spawned successfully");
    Ok(())
}
