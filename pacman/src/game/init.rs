//! SDL initialization, texture setup, ECS event registration, and resource insertion.

use tracing::trace;

use bevy_ecs::event::EventRegistry;
use bevy_ecs::observer::Trigger;
use bevy_ecs::system::ResMut;
use bevy_ecs::world::World;
use sdl2::event::EventType;
use sdl2::image::LoadTexture;
use sdl2::render::{BlendMode, Canvas, ScaleMode, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::asset::Asset;
use crate::constants::{self, CANVAS_SIZE};
use crate::error::{GameError, GameResult};
use crate::events::{CollisionTrigger, GameCommand, GameEvent, StageTransition};
use crate::map::builder::Map;
use crate::platform;
use crate::systems::animation::LinearAnimation;
use crate::systems::audio::{AudioEvent, AudioResource};
use crate::systems::collision::{ghost_collision_observer, item_collision_observer};
use crate::systems::common::{DeltaTime, GlobalState, ScoreResource};
use crate::systems::debug::{BatchedLinesResource, DebugState, DebugTextureResource, TtfAtlasResource};
use crate::systems::hud::FruitSprites;
use crate::systems::input::{Bindings, CursorPosition, TouchState};
use crate::systems::item::PelletCount;
use crate::systems::profiling::{SystemTimings, Timing};
use crate::systems::render::{BackbufferResource, CanvasResource, MapTextureResource, RenderDirty};
use crate::systems::state::{GameStage, PlayerAnimation, PlayerDeathAnimation, PlayerLives};
use crate::systems::state::{IntroPlayed, PauseState};
use crate::texture::sprite::{AtlasMapper, SpriteAtlas};
use crate::texture::sprites::{GameSprite, MazeSprite};

#[cfg(not(target_os = "emscripten"))]
use crate::systems::state::StartupSequence;

pub(super) fn disable_sdl_events(event_pump: &mut EventPump) {
    for event_type in [
        EventType::JoyAxisMotion,
        EventType::JoyBallMotion,
        EventType::JoyHatMotion,
        EventType::JoyButtonDown,
        EventType::JoyButtonUp,
        EventType::JoyDeviceAdded,
        EventType::JoyDeviceRemoved,
        EventType::ControllerAxisMotion,
        EventType::ControllerButtonDown,
        EventType::ControllerButtonUp,
        EventType::ControllerDeviceAdded,
        EventType::ControllerDeviceRemoved,
        EventType::ControllerDeviceRemapped,
        EventType::ControllerTouchpadDown,
        EventType::ControllerTouchpadMotion,
        EventType::ControllerTouchpadUp,
        EventType::DollarGesture,
        EventType::DollarRecord,
        EventType::MultiGesture,
        EventType::ClipboardUpdate,
        EventType::DropFile,
        EventType::DropText,
        EventType::DropBegin,
        EventType::DropComplete,
        EventType::AudioDeviceAdded,
        EventType::AudioDeviceRemoved,
        EventType::RenderTargetsReset,
        EventType::RenderDeviceReset,
        EventType::LocaleChanged,
        EventType::TextInput,
        EventType::TextEditing,
        EventType::Display,
        EventType::MouseWheel,
        EventType::AppDidEnterBackground,
        EventType::AppWillEnterForeground,
        EventType::AppWillEnterBackground,
        EventType::AppDidEnterForeground,
        EventType::AppLowMemory,
        EventType::AppTerminating,
        EventType::User,
        EventType::Last,
    ] {
        event_pump.disable_event(event_type);
    }
}

pub(super) fn setup_textures_and_fonts(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    ttf_context: sdl2::ttf::Sdl2TtfContext,
) -> GameResult<(
    sdl2::render::Texture,
    sdl2::render::Texture,
    sdl2::render::Texture,
    crate::texture::ttf::TtfAtlas,
)> {
    trace!("Creating backbuffer texture");
    let mut backbuffer = texture_creator
        .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
        .map_err(|e| GameError::Sdl(e.to_string()))?;
    backbuffer.set_scale_mode(ScaleMode::Nearest);
    platform::yield_to_browser();

    trace!("Creating map texture");
    let mut map_texture = texture_creator
        .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
        .map_err(|e| GameError::Sdl(e.to_string()))?;
    map_texture.set_scale_mode(ScaleMode::Nearest);
    platform::yield_to_browser();

    trace!("Creating debug texture");
    let output_size = constants::LARGE_CANVAS_SIZE;
    let mut debug_texture = texture_creator
        .create_texture_target(Some(sdl2::pixels::PixelFormatEnum::ARGB8888), output_size.x, output_size.y)
        .map_err(|e| GameError::Sdl(e.to_string()))?;
    debug_texture.set_blend_mode(BlendMode::Blend);
    debug_texture.set_scale_mode(ScaleMode::Nearest);
    platform::yield_to_browser();

    trace!("Loading font");
    let font_data: &'static [u8] = Asset::Font.get_bytes()?.to_vec().leak();
    let font_asset = RWops::from_bytes(font_data).map_err(|_| GameError::Sdl("Failed to load font".to_string()))?;
    let debug_font = ttf_context
        .load_font_from_rwops(font_asset, constants::ui::DEBUG_FONT_SIZE)
        .map_err(|e| GameError::Sdl(e.to_string()))?;

    trace!("Creating TTF atlas");
    let mut ttf_atlas = crate::texture::ttf::TtfAtlas::new(texture_creator, &debug_font)?;
    platform::yield_to_browser();

    trace!("Populating TTF atlas");
    ttf_atlas.populate_atlas(canvas, texture_creator, &debug_font)?;

    Ok((backbuffer, map_texture, debug_texture, ttf_atlas))
}

pub(super) fn load_atlas_and_map_tiles(
    texture_creator: &TextureCreator<WindowContext>,
    atlas_frames: &phf::Map<&'static str, crate::texture::sprite::MapperFrame>,
) -> GameResult<(SpriteAtlas, Vec<crate::texture::sprite::AtlasTile>)> {
    trace!("Loading atlas image from embedded assets");
    let atlas_bytes = Asset::AtlasImage.get_bytes()?;
    let atlas_texture = texture_creator.load_texture_bytes(&atlas_bytes).map_err(|e| {
        if e.to_string().contains("format") || e.to_string().contains("unsupported") {
            GameError::Texture(crate::error::TextureError::InvalidFormat(format!(
                "Unsupported texture format: {e}"
            )))
        } else {
            GameError::Texture(crate::error::TextureError::LoadFailed(e.to_string()))
        }
    })?;

    tracing::debug!(frame_count = atlas_frames.len(), "Creating sprite atlas from texture");
    let atlas_mapper = AtlasMapper {
        frames: atlas_frames.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    };
    let atlas = SpriteAtlas::new(atlas_texture, atlas_mapper);

    trace!("Extracting map tile sprites from atlas");
    let mut map_tiles = Vec::with_capacity(35);
    for i in 0..35 {
        let tile_name = GameSprite::Maze(MazeSprite::Tile(i)).to_path();
        let tile = atlas.get_tile(&tile_name)?;
        map_tiles.push(tile);
    }

    Ok((atlas, map_tiles))
}

pub(super) fn setup_ecs(world: &mut World) {
    EventRegistry::register_event::<GameError>(world);
    EventRegistry::register_event::<GameEvent>(world);
    EventRegistry::register_event::<AudioEvent>(world);
    EventRegistry::register_event::<StageTransition>(world);
    EventRegistry::register_event::<CollisionTrigger>(world);

    world.add_observer(
        |event: Trigger<GameEvent>, mut state: ResMut<GlobalState>, _score: ResMut<ScoreResource>| {
            if matches!(*event, GameEvent::Command(GameCommand::Exit)) {
                state.exit = true;
            }
        },
    );

    world.add_observer(ghost_collision_observer);
    world.add_observer(item_collision_observer);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn insert_resources(
    world: &mut World,
    map: Map,
    audio: crate::audio::Audio,
    atlas: SpriteAtlas,
    event_pump: EventPump,
    canvas: Canvas<Window>,
    backbuffer: sdl2::render::Texture,
    map_texture: sdl2::render::Texture,
    debug_texture: sdl2::render::Texture,
    ttf_atlas: crate::texture::ttf::TtfAtlas,
    death_animation: LinearAnimation,
    red_zones: crate::systems::ghost::RedZoneNodes,
    tunnel_nodes: crate::systems::ghost::TunnelNodes,
) -> GameResult<()> {
    world.insert_non_send_resource(atlas);
    world.insert_resource(super::animations::create_ghost_animations(
        world.non_send_resource::<SpriteAtlas>(),
    )?);
    let player_animation = super::animations::create_player_animations(world.non_send_resource::<SpriteAtlas>())?.0;
    world.insert_resource(PlayerAnimation(player_animation));
    world.insert_resource(PlayerDeathAnimation(death_animation));

    world.insert_resource(FruitSprites::default());
    world.insert_resource(BatchedLinesResource::new(&map, constants::LARGE_SCALE));
    world.insert_resource(map);
    world.insert_resource(GlobalState { exit: false });
    world.insert_resource(PlayerLives::default());
    world.insert_resource(ScoreResource(0));
    world.insert_resource(PelletCount(0));
    world.insert_resource(crate::systems::ghost::GhostModeController::default());
    world.insert_resource(crate::systems::ghost::GhostHouseController::default());
    world.insert_resource(crate::systems::ghost::GhostSpeedConfig::for_level(1));
    world.insert_resource(red_zones);
    world.insert_resource(tunnel_nodes);
    world.insert_resource(SystemTimings::default());
    world.insert_resource(Timing::default());
    world.insert_resource(Bindings::default());
    world.insert_resource(DeltaTime { seconds: 0.0, ticks: 0 });
    world.insert_resource(RenderDirty::default());
    world.insert_resource(DebugState::default());
    world.insert_resource(IntroPlayed::default());
    world.insert_resource(CursorPosition::default());
    world.insert_resource(TouchState::default());
    // On Emscripten, start in WaitingForInteraction state due to browser autoplay policy.
    // The game will transition to Starting when the user clicks or presses a key.
    #[cfg(target_os = "emscripten")]
    world.insert_resource(GameStage::WaitingForInteraction);

    #[cfg(not(target_os = "emscripten"))]
    world.insert_resource(GameStage::Starting(StartupSequence::TextOnly {
        remaining_ticks: constants::startup::STARTUP_FRAMES,
    }));
    world.insert_resource(PauseState::default());

    world.insert_non_send_resource(event_pump);
    world.insert_non_send_resource(CanvasResource(canvas));
    world.insert_non_send_resource(BackbufferResource(backbuffer));
    world.insert_non_send_resource(MapTextureResource(map_texture));
    world.insert_non_send_resource(DebugTextureResource(debug_texture));
    world.insert_non_send_resource(TtfAtlasResource(ttf_atlas));
    world.insert_non_send_resource(AudioResource(audio));
    Ok(())
}
