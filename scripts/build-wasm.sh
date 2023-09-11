#!/bin/sh
set -eux

cargo build --target=wasm32-unknown-emscripten --release

mkdir -p dist

cp target/wasm32-unknown-emscripten/release/pacman.wasm dist
cp target/wasm32-unknown-emscripten/release/pacman.js dist
cp target/wasm32-unknown-emscripten/release/deps/pacman.data dist/deps
cp assets/index.html dist