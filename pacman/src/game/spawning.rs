//! Entity spawning for player, ghosts, and collectible items.

use tracing::{info, trace};

use bevy_ecs::world::World;

use crate::constants;
use crate::constants::MapTile;
use crate::error::GameResult;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::animation::{Blinking, DirectionalAnimation};
use crate::systems::collision::{Collider, GhostCollider, ItemCollider, PacmanCollider};
use crate::systems::common::{EntityType, Frozen, GhostBundle, ItemBundle, MovementModifiers, PlayerBundle};
use crate::systems::ghost::{GhostAnimationState, GhostAnimations, GhostType, LastAnimationState};
use crate::systems::movement::{BufferedDirection, NodeId, Position, Velocity};
use crate::systems::player::PlayerControlled;
use crate::systems::render::{Renderable, Visibility};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{GameSprite, GhostSprite, MazeSprite};

pub(super) fn create_player_bundle(
    map: &Map,
    player_animation: DirectionalAnimation,
    player_start_sprite: AtlasTile,
) -> PlayerBundle {
    PlayerBundle {
        player: PlayerControlled,
        position: Position::Stopped {
            node: map.start_positions.pacman,
        },
        velocity: Velocity {
            speed: constants::mechanics::PLAYER_SPEED,
            direction: Direction::Left,
        },
        movement_modifiers: MovementModifiers::default(),
        buffered_direction: BufferedDirection::None,
        sprite: Renderable {
            sprite: player_start_sprite,
            layer: 0,
        },
        directional_animation: player_animation,
        entity_type: EntityType::Player,
        collider: Collider {
            size: constants::collider::PLAYER_SIZE,
        },
        pacman_collider: PacmanCollider,
    }
}

pub(super) fn spawn_items(world: &mut World) -> GameResult<()> {
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
        let mut item = world.spawn(ItemBundle {
            position: Position::Stopped { node: id },
            sprite: Renderable { sprite, layer: 1 },
            entity_type: item_type,
            collider: Collider { size },
            item_collider: ItemCollider,
        });

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
pub(super) fn spawn_ghosts(world: &mut World) -> GameResult<()> {
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

            let bundle = GhostBundle {
                ghost: ghost_type,
                position: Position::Stopped { node: start_node },
                velocity: Velocity {
                    speed: ghost_type.base_speed(),
                    direction: Direction::Left,
                },
                sprite: Renderable {
                    sprite: SpriteAtlas::get_tile(atlas, &sprite_path)?,
                    layer: 0,
                },
                directional_animation: animations,
                entity_type: EntityType::Ghost,
                collider: Collider {
                    size: constants::collider::GHOST_SIZE,
                },
                ghost_collider: GhostCollider,
                ghost_state,
                last_animation_state: LastAnimationState(GhostAnimationState::Normal),
                ghost_target: crate::systems::ghost::GhostTarget::default(),
            };

            // Blinky gets additional components
            let extra = if ghost_type == GhostType::Blinky {
                Some((crate::systems::ghost::BlinkyMarker, crate::systems::ghost::Elroy::default()))
            } else {
                None
            };

            (bundle, extra)
        };

        let mut entity_commands = world.spawn(ghost_bundle);
        entity_commands.insert((Frozen, Visibility::hidden()));

        if let Some((marker, elroy)) = extra_components {
            entity_commands.insert((marker, elroy));
        }

        let entity = entity_commands.id();
        trace!(ghost = ?ghost_type, entity = ?entity, start_node, "Spawned ghost entity");
    }

    info!("All ghost entities spawned successfully");
    Ok(())
}
