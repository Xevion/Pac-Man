use assert2::check;
use pacman::asset::Asset;

#[test]
fn all_asset_paths_exist() {
    for asset in Asset::into_iter() {
        let path = asset.path();
        let full_path = format!("assets/game/{}", path);

        let metadata = std::fs::metadata(&full_path)
            .map_err(|e| format!("Error getting metadata for {}: {}", full_path, e))
            .unwrap();
        check!(metadata.is_file());
        check!(metadata.len() > 1024);
    }
}
