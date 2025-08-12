use pacman::asset::Asset;
use std::path::Path;
use strum::IntoEnumIterator;

#[test]
fn test_asset_paths_valid() {
    let base_path = Path::new("assets/game/");

    for asset in Asset::iter() {
        let path = base_path.join(asset.path());
        assert!(path.exists(), "Asset path does not exist: {:?}", path);
        assert!(path.is_file(), "Asset path is not a file: {:?}", path);
    }
}
