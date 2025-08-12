use crate::{
    constants,
    entity::graph::Graph,
    error::EntityError,
    texture::sprite::{Sprite, SpriteAtlas},
};
use sdl2::render::{Canvas, RenderTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Pellet,
    Energizer,
    #[allow(dead_code)]
    Fruit {
        kind: FruitKind,
    },
}

impl ItemType {
    pub fn get_score(self) -> u32 {
        match self {
            ItemType::Pellet => 10,
            ItemType::Energizer => 50,
            ItemType::Fruit { kind } => kind.get_score(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FruitKind {
    Apple,
    Strawberry,
    Orange,
    Melon,
    Bell,
    Key,
    Galaxian,
}

impl FruitKind {
    pub fn get_score(self) -> u32 {
        match self {
            FruitKind::Apple => 100,
            FruitKind::Strawberry => 300,
            FruitKind::Orange => 500,
            FruitKind::Melon => 700,
            FruitKind::Bell => 1000,
            FruitKind::Key => 2000,
            FruitKind::Galaxian => 3000,
        }
    }
}

pub struct Item {
    pub node_index: usize,
    pub item_type: ItemType,
    pub sprite: Sprite,
    pub collected: bool,
}

impl Item {
    pub fn new(node_index: usize, item_type: ItemType, sprite: Sprite) -> Self {
        Self {
            node_index,
            item_type,
            sprite,
            collected: false,
        }
    }

    pub fn is_collected(&self) -> bool {
        self.collected
    }

    pub fn collect(&mut self) {
        self.collected = true;
    }

    pub fn get_score(&self) -> u32 {
        self.item_type.get_score()
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, graph: &Graph) -> anyhow::Result<()> {
        if !self.collected {
            let node = graph
                .get_node(self.node_index)
                .ok_or(EntityError::NodeNotFound(self.node_index))?;
            let position = node.position + constants::BOARD_PIXEL_OFFSET.as_vec2();
            self.sprite.render(canvas, atlas, position)
        } else {
            Ok(())
        }
    }
}
