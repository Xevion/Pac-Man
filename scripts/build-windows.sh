#!/bin/bash
set -eu

SDL_VERSION="2.28.3"
SDL_IMAGE_VERSION="2.6.3"
SDL_MIXER_VERSION="2.6.3"
SDL_TTF_VERSION="2.20.2"

SDL="https://github.com/libsdl-org/SDL/releases/download/release-${SDL_VERSION}/SDL2-devel-${SDL_VERSION}-mingw.tar.gz"
SLD_IMAGE="https://github.com/libsdl-org/SDL_image/releases/download/release-${SDL_IMAGE_VERSION}/SDL2_image-devel-${SDL_IMAGE_VERSION}-mingw.tar.gz"
SDL_MIXER="https://github.com/libsdl-org/SDL_mixer/releases/download/release-${SDL_MIXER_VERSION}/SDL2_mixer-devel-${SDL_MIXER_VERSION}-mingw.tar.gz"
SDL_TTF="https://github.com/libsdl-org/SDL_ttf/releases/download/release-${SDL_TTF_VERSION}/SDL_ttf-devel-${SDL_TTF_VERSION}-mingw.tar.gz"

# Verify that toolchain is installed


# EXTRACT_DIR="~/.rustup/"
EXTRACT_DIR="~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/"
# if [ ! -d "~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/" ]; then
    # ls $EXTRACT_DIR
    # echo "Toolchain not installed. Run rustup target add x86_64-pc-windows-gnu. This may not work on environments besides Linux GNU."
    # exit 1
# fi

echo "Downloading..."
curl -L -o ./sdl2.tar.gz $SDL
curl -L -o ./sdl2_image.tar.gz $SLD_IMAGE
echo "Done."