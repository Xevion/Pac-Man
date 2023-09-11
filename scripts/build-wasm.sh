#!/bin/sh
set -eux


echo "Building WASM with Emscripten"
cargo build --target=wasm32-unknown-emscripten --release

echo "Copying release files to dist/"
mkdir -p dist
mkdir -p dist/deps

output_folder="target/wasm32-unknown-emscripten/release"
cp $output_folder/pacman.wasm dist
cp $output_folder/pacman.js dist
cp $output_folder/deps/pacman.data dist/deps
cp assets/index.html dist