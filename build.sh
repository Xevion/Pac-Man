#!/bin/bash
set -eu

release='false'
serve='false'
skip_emsdk='false'
clean='false'

print_usage() {
    printf "Usage: -erdsc\n"
    printf "  -e: Skip EMSDK setup (GitHub workflow only)\n"
    printf "  -r: Build in release mode\n"
    printf "  -d: Build in debug mode\n"
    printf "  -s: Serve the WASM files once built\n"
    printf "  -c: Clean the target/dist directory\n"
}

while getopts 'erdsc' flag; do
    case "${flag}" in
    e) skip_emsdk='true' ;;
    r) release='true' ;;
    d) release='false' ;; # doesn't actually do anything, but last flag wins
    s) serve='true' ;;
    c) clean='true' ;;
    *)
        print_usage
        exit 1
        ;;
    esac
done

if [ "$clean" = 'true' ]; then
    echo "Cleaning target directory"
    cargo clean
    rm -rf ./dist/
fi

if [ "$skip_emsdk" = 'false' ]; then
    echo "Activating Emscripten"
    # SDL2-TTF requires 3.1.43, fails to build on latest
    ../emsdk/emsdk activate 3.1.43
    source ../emsdk/emsdk_env.sh
fi

echo "Building WASM with Emscripten"
build_type='debug'
if [ "$release" = 'true' ]; then
    cargo build --target=wasm32-unknown-emscripten --release
    build_type='release'
else
    cargo build --target=wasm32-unknown-emscripten
fi

echo "Generating CSS"
pnpx postcss-cli ./assets/styles.scss -o ./assets/build.css

echo "Copying WASM files"
mkdir -p dist
output_folder="target/wasm32-unknown-emscripten/$build_type"
cp assets/index.html dist

# cp assets/*.woff* dist
cp assets/build.css dist
cp assets/favicon.ico dist
cp $output_folder/pacman.wasm dist
cp $output_folder/pacman.js dist
# only if .data file exists
cp $output_folder/deps/pacman.data dist
if [ -f $output_folder/pacman.wasm.map ]; then
    cp $output_folder/pacman.wasm.map dist
fi

if [ "$serve" = 'true' ]; then
    echo "Serving WASM with Emscripten"
    python3 -m http.server -d ./dist/ 8080
fi
