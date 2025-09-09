#![allow(dead_code)]

use bevy_ecs::{entity::Entity, event::Events, world::World};
use glam::{U16Vec2, Vec2};
use pacman::{
    asset::{get_asset_bytes, Asset},
    constants::RAW_BOARD,
    events::GameEvent,
    game::ATLAS_FRAMES,
    map::{
        builder::Map,
        direction::Direction,
        graph::{Graph, Node},
    },
    systems::{
        AudioEvent, AudioState, BufferedDirection, Collider, DebugState, DeltaTime, EntityType, Ghost, GhostCollider, GhostState,
        GlobalState, ItemCollider, MovementModifiers, PacmanCollider, PelletCount, PlayerControlled, Position, ScoreResource,
        Velocity,
    },
    texture::sprite::{AtlasMapper, AtlasTile, SpriteAtlas},
};
use sdl2::{
    image::LoadTexture,
    pixels::Color,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

pub fn setup_sdl() -> Result<(Canvas<Window>, TextureCreator<WindowContext>, Sdl), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("test", 800, 600)
        .position_centered()
        .hidden()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    Ok((canvas, texture_creator, sdl_context))
}

pub fn create_atlas(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> SpriteAtlas {
    let texture_creator = canvas.texture_creator();
    let atlas_bytes = get_asset_bytes(Asset::AtlasImage).unwrap();

    let texture = texture_creator.load_texture_bytes(&atlas_bytes).unwrap();

    let atlas_mapper = AtlasMapper {
        frames: ATLAS_FRAMES.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    };

    SpriteAtlas::new(texture, atlas_mapper)
}

/// Creates a simple test graph with 3 connected nodes for testing
pub fn create_test_graph() -> Graph {
    let mut graph = Graph::new();

    let node0 = graph.add_node(Node {
        position: Vec2::new(0.0, 0.0),
    });
    let node1 = graph.add_node(Node {
        position: Vec2::new(16.0, 0.0),
    });
    let node2 = graph.add_node(Node {
        position: Vec2::new(0.0, 16.0),
    });

    graph.connect(node0, node1, false, None, Direction::Right).unwrap();
    graph.connect(node0, node2, false, None, Direction::Down).unwrap();

    graph
}

/// Creates a basic test world with required resources for ECS systems
pub fn create_test_world() -> World {
    let mut world = World::new();

    // Add required resources
    world.insert_resource(Events::<GameEvent>::default());
    world.insert_resource(Events::<pacman::error::GameError>::default());
    world.insert_resource(Events::<AudioEvent>::default());
    world.insert_resource(ScoreResource(0));
    world.insert_resource(AudioState::default());
    world.insert_resource(GlobalState { exit: false });
    world.insert_resource(DebugState::default());
    world.insert_resource(PelletCount(0));
    world.insert_resource(DeltaTime {
        seconds: 1.0 / 60.0,
        ticks: 1,
    }); // 60 FPS
    world.insert_resource(create_test_map());

    world
}

/// Creates a test map using the default RAW_BOARD
pub fn create_test_map() -> Map {
    Map::new(RAW_BOARD).expect("Failed to create test map")
}

/// Spawns a test Pac-Man entity at the specified node
pub fn spawn_test_pacman(world: &mut World, node: usize) -> Entity {
    world
        .spawn((
            Position::Stopped { node: node as u16 },
            Collider { size: 10.0 },
            PacmanCollider,
            EntityType::Player,
        ))
        .id()
}

/// Spawns a controllable test player entity
pub fn spawn_test_player(world: &mut World, node: usize) -> Entity {
    world
        .spawn((
            PlayerControlled,
            Position::Stopped { node: node as u16 },
            Velocity {
                speed: 1.0,
                direction: Direction::Right,
            },
            BufferedDirection::None,
            EntityType::Player,
            MovementModifiers::default(),
        ))
        .id()
}

/// Spawns a test item entity at the specified node
pub fn spawn_test_item(world: &mut World, node: usize, item_type: EntityType) -> Entity {
    world
        .spawn((
            Position::Stopped { node: node as u16 },
            Collider { size: 8.0 },
            ItemCollider,
            item_type,
        ))
        .id()
}

/// Spawns a test ghost entity at the specified node
pub fn spawn_test_ghost(world: &mut World, node: usize, ghost_state: GhostState) -> Entity {
    world
        .spawn((
            Position::Stopped { node: node as u16 },
            Collider { size: 12.0 },
            GhostCollider,
            Ghost::Blinky,
            EntityType::Ghost,
            ghost_state,
        ))
        .id()
}

/// Sends a game event to the world
pub fn send_game_event(world: &mut World, event: GameEvent) {
    let mut events = world.resource_mut::<Events<GameEvent>>();
    events.send(event);
}

/// Sends a collision event between two entities
pub fn send_collision_event(world: &mut World, entity1: Entity, entity2: Entity) {
    let mut events = world.resource_mut::<Events<GameEvent>>();
    events.send(GameEvent::Collision(entity1, entity2));
}

/// Creates a mock atlas tile for testing
pub fn mock_atlas_tile(id: u32) -> AtlasTile {
    AtlasTile {
        pos: U16Vec2::new(0, 0),
        size: U16Vec2::new(16, 16),
        color: Some(Color::RGB(id as u8, 0, 0)),
    }
}
