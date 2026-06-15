use crate::systems::item::FruitType;
use bevy_ecs::resource::Resource;

/// Collected-fruit history shown in the HUD, in collection order (oldest first).
/// The chrome renderer displays the most recent few, right-aligned.
#[derive(Resource, Default)]
pub struct FruitSprites(pub Vec<FruitType>);
