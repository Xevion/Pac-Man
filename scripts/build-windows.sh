#!/bin/bash
set -eu

SDL_VERSION="2.28.3"
SDL_IMAGE_VERSION="2.6.3"
SDL_MIXER_VERSION="2.6.3"
SDL_TTF_VERSION="2.20.2"

SDL="https://github.com/libsdl-org/SDL/releases/download/release-${SDL_VERSION}/SDL2-devel-${SDL_VERSION}-mingw.tar.gz"
SLD_IMAGE="https://github.com/libsdl-org/SDL_image/releases/download/release-${SDL_IMAGE_VERSION}/SDL2_image-devel-${SDL_IMAGE_VERSION}-mingw.tar.gz"
SDL_MIXER="https://github.com/libsdl-org/SDL_mixer/releases/download/release-${SDL_MIXER_VERSION}/SDL2_mixer-devel-${SDL_MIXER_VERSION}-mingw.tar.gz"
SDL_TTF="https://github.com/libsdl-org/SDL_ttf/releases/download/release-${SDL_TTF_VERSION}/SDL2_ttf-devel-${SDL_TTF_VERSION}-mingw.tar.gz"

EXTRACT_DIR="./target/x86_64-pc-windows-gnu/release/deps"

if [ ! -f $EXTRACT_DIR/libSDL2.a ]; then
    if [ ! -f ./sdl2.tar.gz ]; then
        echo "Downloading SDL2@$SDL_VERSION..."
        curl -L -o ./sdl2.tar.gz $SDL
    fi
    echo "Extracting SDL2..."
    tar -xzf ./sdl2.tar.gz -C $EXTRACT_DIR --strip-components=3 "SDL2-$SDL_VERSION/x86_64-w64-mingw32/lib/libSDL2.a"
    rm -f ./sdl2.tar.gz
fi

if [ ! -f $EXTRACT_DIR/libSDL2_image.a ]; then
    if [ ! -f ./sdl2_image.tar.gz ]; then
        echo "Downloading SDL2_image@$SDL_IMAGE_VERSION..."
        curl -L -o ./sdl2_image.tar.gz $SLD_IMAGE
    fi
    echo "Extracting SDL2_image..."
    tar -xzf ./sdl2_image.tar.gz -C $EXTRACT_DIR --strip-components=3 "SDL2_image-$SDL_IMAGE_VERSION/x86_64-w64-mingw32/lib/libSDL2_image.a"
fi
rm -f ./sdl2_image.tar.gz

if [ ! -f $EXTRACT_DIR/libSDL2_mixer.a ]; then
    if [ ! -f ./sdl2_mixer.tar.gz ]; then
        echo "Downloading SDL2_mixer@$SDL_MIXER_VERSION..."
        curl -L -o ./sdl2_mixer.tar.gz $SDL_MIXER
    fi
    echo "Extracting SDL2_mixer..."
    tar -xzf ./sdl2_mixer.tar.gz -C $EXTRACT_DIR --strip-components=3 "SDL2_mixer-$SDL_MIXER_VERSION/x86_64-w64-mingw32/lib/libSDL2_mixer.a"
    rm -f ./sdl2_mixer.tar.gz
fi

if [ ! -f $EXTRACT_DIR/libSDL2_ttf.a ]; then
    
    if [ ! -f ./sdl2_ttf.tar.gz ]; then
        echo "Downloading SDL2_ttf@$SDL_TTF_VERSION..."
        curl -L -o ./sdl2_ttf.tar.gz $SDL_TTF
    fi
    echo "Extracting SDL2_ttf..."
    tar -xzf ./sdl2_ttf.tar.gz -C $EXTRACT_DIR --strip-components=3 "SDL2_ttf-$SDL_TTF_VERSION/x86_64-w64-mingw32/lib/libSDL2_ttf.a"
    rm -f ./sdl2_ttf.tar.gz
fi

echo "Building..."
cargo zigbuild --release --target x86_64-pc-windows-gnu