use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AtlasMapper {
    frames: HashMap<String, MapperFrame>,
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct MapperFrame {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("atlas_data.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let atlas_json = include_str!("./assets/game/atlas.json");
    let atlas_mapper: AtlasMapper = serde_json::from_str(atlas_json).unwrap();

    writeln!(&mut file, "use phf::phf_map;").unwrap();

    writeln!(&mut file, "use crate::texture::sprite::MapperFrame;").unwrap();

    writeln!(
        &mut file,
        "pub static ATLAS_FRAMES: phf::Map<&'static str, MapperFrame> = phf_map! {{"
    )
    .unwrap();

    for (name, frame) in atlas_mapper.frames {
        writeln!(
            &mut file,
            "    \"{}\" => MapperFrame {{ x: {}, y: {}, width: {}, height: {} }},",
            name, frame.x, frame.y, frame.width, frame.height
        )
        .unwrap();
    }

    writeln!(&mut file, "}};").unwrap();
    println!("cargo:rerun-if-changed=assets/game/atlas.json");
}
