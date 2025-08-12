use glam::U16Vec2;
use pacman::{
    entity::{
        collision::Collidable,
        item::{FruitKind, Item, ItemType},
    },
    texture::sprite::{AtlasTile, Sprite},
};
use strum::{EnumCount, IntoEnumIterator};

#[test]
fn test_item_type_get_score() {
    assert_eq!(ItemType::Pellet.get_score(), 10);
    assert_eq!(ItemType::Energizer.get_score(), 50);

    let fruit = ItemType::Fruit { kind: FruitKind::Apple };
    assert_eq!(fruit.get_score(), 100);
}

#[test]
fn test_fruit_kind_increasing_score() {
    // Build a list of fruit kinds, sorted by their index
    let mut kinds = FruitKind::iter()
        .map(|kind| (kind.index(), kind.get_score()))
        .collect::<Vec<_>>();
    kinds.sort_unstable_by_key(|(index, _)| *index);

    assert_eq!(kinds.len(), FruitKind::COUNT as usize);

    // Check that the score increases as expected
    for window in kinds.windows(2) {
        let ((_, prev), (_, next)) = (window[0], window[1]);
        assert!(prev < next, "Fruits should have increasing scores, but {prev:?} < {next:?}");
    }
}

#[test]
fn test_item_creation_and_collection() {
    let atlas_tile = AtlasTile {
        pos: U16Vec2::new(0, 0),
        size: U16Vec2::new(16, 16),
        color: None,
    };
    let sprite = Sprite::new(atlas_tile);
    let mut item = Item::new(0, ItemType::Pellet, sprite);

    assert!(!item.is_collected());
    assert_eq!(item.get_score(), 10);
    assert_eq!(item.position().from_node_id(), 0);

    item.collect();
    assert!(item.is_collected());
}
