# Pac-Man

If the title doesn't clue you in, I'm remaking Pac-Man with SDL and Rust.

The project is _extremely_ early in development, but check back in a week, and maybe I'll have something cool to look
at.

## Feature Targets

- Near-perfect replication of logic, scoring, graphics, sound, and behaviors.
- Written in Rust, buildable on Windows, Linux, Mac and WebAssembly.
- Online demo, playable in a browser.
- Automatic build system, with releases for Windows, Linux, and Mac & Web-Assembly.
- Debug tooling
    - Game state visualization
    - Game speed controls + pausing
    - Log tracing
    - Performance details

## Experimental Ideas

- Perfected Ghost Algorithms
- More than 4 ghosts
- Custom Level Generation
    - Multi-map tunnelling

## Installation

Besides SDL2, the following extensions are required: Image, Mixer, and TTF.

### Ubuntu

On Ubuntu, you can install the required packages with the following command:

```
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-mixer-dev libsdl2-ttf-dev
```

### Windows

On Windows, installation requires either building from source (not covered), or downloading the pre-built binaries.

The latest releases can be found here:

- [SDL2](https://github.com/libsdl-org/SDL/releases/latest/)
- [SDL2_image](https://github.com/libsdl-org/SDL_image/releases/latest/)
- [SDL2_mixer](https://github.com/libsdl-org/SDL_mixer/releases/latest/)
- [SDL2_ttf](https://github.com/libsdl-org/SDL_ttf/releases/latest/)

Download each for your architecture, and locate the appropriately named DLL within. Move said DLL to root of this project.

In total, you should have the following DLLs in the root of the project:

- SDL2.dll
- SDL2_mixer.dll
- SDL2_ttf.dll
- SDL2_image.dll
- libpngX-X.dll
    - Not sure on what specific version is to be used, or if naming matters. `libpng16-16.dll` is what I had used.
- zlib1.dll

## Building

To build the project, run the following command:

```
cargo build
```

During development, you can easily run the project with:

```
cargo run
cargo run -q # Quiet mode, no logging
cargo run --release # Release mode, optimized
```