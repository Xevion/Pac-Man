& cargo build --target=wasm32-unknown-emscripten --release

mkdir -p dist -Force

cp ./target/wasm32-unknown-emscripten/release/Pac_Man.wasm ./dist
cp ./target/wasm32-unknown-emscripten/release/Pac-Man.js ./dist
cp index.html dist