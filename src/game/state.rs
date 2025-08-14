use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};
use smallvec::SmallVec;

use crate::{
    asset::{get_asset_bytes, Asset},
    audio::Audio,
    constants::RAW_BOARD,
    entity::{
        collision::{Collidable, CollisionSystem},
        ghost::{Ghost, GhostType},
        item::Item,
        pacman::Pacman,
    },
    error::{GameError, GameResult, TextureError},
    game::EntityId,
    map::Map,
    texture::{
        sprite::{AtlasMapper, SpriteAtlas},
        text::TextTexture,
    },
};

include!(concat!(env!("OUT_DIR"), "/atlas_data.rs"));

/// The `GameState` struct holds all the essential data for the game.
///
/// This includes the score, map, entities (Pac-Man, ghosts, items),
/// collision system, and rendering resources. By centralizing the game's state,
/// we can cleanly separate it from the game's logic, making it easier to manage
/// and reason about.
pub struct GameState {
    pub score: u32,
    pub map: Map,
    pub pacman: Pacman,
    pub pacman_id: EntityId,
    pub ghosts: SmallVec<[Ghost; 4]>,
    pub ghost_ids: SmallVec<[EntityId; 4]>,
    pub items: Vec<Item>,
    pub item_ids: Vec<EntityId>,
    pub debug_mode: bool,

    // Collision system
    pub(crate) collision_system: CollisionSystem,

    // Rendering resources
    pub(crate) atlas: SpriteAtlas,
    pub(crate) text_texture: TextTexture,

    // Audio
    pub audio: Audio,

    // Map texture pre-rendering
    pub(crate) map_texture: Option<Texture<'static>>,
    pub(crate) map_rendered: bool,
    pub(crate) texture_creator: &'static TextureCreator<WindowContext>,
}

impl GameState {
    /// Creates a new `GameState` by initializing all the game's data.
    ///
    /// This function sets up the map, Pac-Man, ghosts, items, collision system,
    /// and all rendering resources required to start the game. It returns a `GameResult`
    /// to handle any potential errors during initialization.
    pub fn new(texture_creator: &'static TextureCreator<WindowContext>) -> GameResult<Self> {
        let map = Map::new(RAW_BOARD)?;

        let start_node = map.start_positions.pacman;

        let atlas_bytes = get_asset_bytes(Asset::Atlas)?;
        let atlas_texture = texture_creator.load_texture_bytes(&atlas_bytes).map_err(|e| {
            if e.to_string().contains("format") || e.to_string().contains("unsupported") {
                GameError::Texture(TextureError::InvalidFormat(format!("Unsupported texture format: {e}")))
            } else {
                GameError::Texture(TextureError::LoadFailed(e.to_string()))
            }
        })?;

        let atlas_mapper = AtlasMapper {
            frames: ATLAS_FRAMES.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        };
        let atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

        let text_texture = TextTexture::new(1.0);
        let audio = Audio::new();
        let pacman = Pacman::new(&map.graph, start_node, &atlas)?;

        // Generate items (pellets and energizers)
        let items = map.generate_items(&atlas)?;

        // Initialize collision system
        let mut collision_system = CollisionSystem::default();

        // Register Pac-Man
        let pacman_id = collision_system.register_entity(pacman.position());

        // Register items
        let item_ids = items
            .iter()
            .map(|item| collision_system.register_entity(item.position()))
            .collect();

        // Create and register ghosts
        let ghosts = [GhostType::Blinky, GhostType::Pinky, GhostType::Inky, GhostType::Clyde]
            .iter()
            .zip(
                [
                    map.start_positions.blinky,
                    map.start_positions.pinky,
                    map.start_positions.inky,
                    map.start_positions.clyde,
                ]
                .iter(),
            )
            .map(|(ghost_type, start_node)| Ghost::new(&map.graph, *start_node, *ghost_type, &atlas))
            .collect::<GameResult<SmallVec<[_; 4]>>>()?;

        // Register ghosts
        let ghost_ids = ghosts
            .iter()
            .map(|ghost| collision_system.register_entity(ghost.position()))
            .collect();

        Ok(Self {
            map,
            atlas,
            pacman,
            pacman_id,
            ghosts,
            ghost_ids,
            items,
            item_ids,
            text_texture,
            audio,
            score: 0,
            debug_mode: false,
            collision_system,
            map_texture: None,
            map_rendered: false,
            texture_creator,
        })
    }
}
