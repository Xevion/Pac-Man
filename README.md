# Pac-Man

[![Tests Status][badge-test]][test] [![Build Status][badge-build]][build] [![Code Coverage][badge-coverage]][coverage] [![Online Demo][badge-online-demo]][demo] [![Last Commit][badge-last-commit]][commits]

[badge-test]: https://github.com/Xevion/Pac-Man/actions/workflows/test.yaml/badge.svg
[badge-build]: https://github.com/Xevion/Pac-Man/actions/workflows/build.yaml/badge.svg
[badge-coverage]: https://coveralls.io/repos/github/Xevion/Pac-Man/badge.svg?branch=master
[badge-demo]: https://img.shields.io/github/deployments/Xevion/Pac-Man/github-pages?label=GitHub%20Pages
[badge-online-demo]: https://img.shields.io/badge/GitHub%20Pages-Demo-brightgreen
[badge-last-commit]: https://img.shields.io/github/last-commit/Xevion/Pac-Man
[build]: https://github.com/Xevion/Pac-Man/actions/workflows/build.yaml
[test]: https://github.com/Xevion/Pac-Man/actions/workflows/test.yaml
[coverage]: https://coveralls.io/github/Xevion/Pac-Man?branch=master
[demo]: https://xevion.github.io/Pac-Man/
[commits]: https://github.com/Xevion/Pac-Man/commits/master

## Description

A faithful recreation of the classic Pac-Man arcade game written in Rust. This project aims to replicate the original game's mechanics, graphics, sound, and behavior as accurately as possible while providing modern development features like cross-platform compatibility and WebAssembly support.

The game includes all the original features you'd expect from Pac-Man:

- [x] Classic maze navigation and dot collection
- [ ] Four ghosts with their unique AI behaviors (Blinky, Pinky, Inky, and Clyde)
- [ ] Power pellets that allow Pac-Man to eat ghosts
- [ ] Fruit bonuses that appear periodically
- [ ] Progressive difficulty with faster ghosts and shorter power pellet duration
- [x] Authentic sound effects and sprites

Built with SDL2 for cross-platform graphics and audio, this implementation can run on Windows, Linux, macOS, and in web browsers via WebAssembly.

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
- Online Scoreboard
  - WebAssembly build contains a special API key for communicating with server.
  - To prevent abuse, the server will only accept scores from the WebAssembly build.

## Build Notes

- Install `cargo-vcpkg` with `cargo install cargo-vcpkg`, then run `cargo vcpkg build` to build the requisite dependencies via vcpkg.
- For the WASM build, you need to have the Emscripten SDK cloned; you can do so with `git clone https://github.com/emscripten-core/emsdk.git`
  - The first time you clone, you'll need to install the appropriate SDK version with `./emsdk install 3.1.43` and then activate it with `./emsdk activate 3.1.43`. On Windows, use `./emsdk/emsdk.ps1` instead.
  - You can then activate the Emscripten SDK with `source ./emsdk/emsdk_env.sh` or `./emsdk/emsdk_env.ps1` or `./emsdk/emsdk_env.bat` depending on your OS/terminal.
  - While using the `web.build.ts` is not technically required, it simplifies the build process and is very helpful.
    - It is intended to be run with `bun`, which you can acquire at [bun.sh](https://bun.sh/)
  - Tip: You can launch a fileserver with `python` or `caddy` to serve the files in the `dist` folder.
    - `python3 -m http.server 8080 -d dist`
    - `caddy file-server --root dist` (install with `[sudo apt|brew|choco] install caddy` or [a dozen other ways](https://caddyserver.com/docs/install))
