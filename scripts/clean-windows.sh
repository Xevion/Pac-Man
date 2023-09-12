#!/bin/bash
set -eux

echo "Cleaning library files from ./target/x86_64-pc-windows-gnu/release/deps"
rm -f ./target/x86_64-pc-windows-gnu/release/deps/libSDL2.a
rm -f ./target/x86_64-pc-windows-gnu/release/deps/libSDL2_image.a
rm -f ./target/x86_64-pc-windows-gnu/release/deps/libSDL2_mixer.a
rm -f ./target/x86_64-pc-windows-gnu/release/deps/libSDL2_ttf.a
echo "Done."