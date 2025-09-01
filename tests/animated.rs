// use glam::U16Vec2;
// use pacman::error::{AnimatedTextureError, GameError, TextureError};
// use pacman::texture::sprite::AtlasTile;
// use sdl2::pixels::Color;
// use smallvec::smallvec;

// fn mock_atlas_tile(id: u32) -> AtlasTile {
//     AtlasTile {
//         pos: U16Vec2::new(0, 0),
//         size: U16Vec2::new(16, 16),
//         color: Some(Color::RGB(id as u8, 0, 0)),
//     }
// }

// #[test]
// fn test_animated_texture_creation_errors() {
//     let tiles = smallvec![mock_atlas_tile(1), mock_atlas_tile(2)];

//     assert!(matches!(
//         AnimatedTexture::new(tiles.clone(), 0).unwrap_err(),
//         GameError::Texture(TextureError::Animated(AnimatedTextureError::InvalidFrameDuration(0)))
//     ));
// }

// #[test]
// fn test_animated_texture_advancement() {
//     let tiles = smallvec![mock_atlas_tile(1), mock_atlas_tile(2), mock_atlas_tile(3)];
//     let mut texture = AnimatedTexture::new(tiles, 10).unwrap();

//     assert_eq!(texture.current_frame(), 0);

//     texture.tick(25);
//     assert_eq!(texture.current_frame(), 2);
//     assert_eq!(texture.time_bank(), 5);
// }

// #[test]
// fn test_animated_texture_wrap_around() {
//     let tiles = smallvec![mock_atlas_tile(1), mock_atlas_tile(2)];
//     let mut texture = AnimatedTexture::new(tiles, 10).unwrap();

//     texture.tick(10);
//     assert_eq!(texture.current_frame(), 1);

//     texture.tick(10);
//     assert_eq!(texture.current_frame(), 0);
// }

// #[test]
// fn test_animated_texture_single_frame() {
//     let tiles = smallvec![mock_atlas_tile(1)];
//     let mut texture = AnimatedTexture::new(tiles, 10).unwrap();

//     texture.tick(10);
//     assert_eq!(texture.current_frame(), 0);
//     assert_eq!(texture.current_tile().color.unwrap().r, 1);
// }
