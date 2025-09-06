use pacman::asset::Asset;
use speculoos::prelude::*;
use strum::IntoEnumIterator;

#[test]
fn all_asset_paths_exist() {
    for asset in Asset::iter() {
        let path = asset.path();
        let full_path = format!("assets/game/{}", path);

        let metadata = std::fs::metadata(&full_path)
            .map_err(|e| format!("Error getting metadata for {}: {}", full_path, e))
            .unwrap();
        assert_that(&metadata.is_file()).is_true();
        assert_that(&metadata.len()).is_greater_than(1024);
    }
}
