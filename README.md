# Pac-Man

[![Tests Status][badge-test]][test] [![Build Status][badge-build]][build] [![Code Coverage][badge-coverage]][coverage] [![Online Demo][badge-online-demo]][demo] [![Last Commit][badge-last-commit]][commits]

[badge-test]: https://github.com/Xevion/Pac-Man/actions/workflows/tests.yaml/badge.svg
[badge-build]: https://github.com/Xevion/Pac-Man/actions/workflows/build.yaml/badge.svg
[badge-coverage]: https://coveralls.io/repos/github/Xevion/Pac-Man/badge.svg?branch=master
[badge-demo]: https://img.shields.io/github/deployments/Xevion/Pac-Man/github-pages?label=GitHub%20Pages
[badge-online-demo]: https://img.shields.io/badge/GitHub%20Pages-Demo-brightgreen
[badge-last-commit]: https://img.shields.io/github/last-commit/Xevion/Pac-Man
[build]: https://github.com/Xevion/Pac-Man/actions/workflows/build.yaml
[test]: https://github.com/Xevion/Pac-Man/actions/workflows/tests.yaml
[coverage]: https://coveralls.io/github/Xevion/Pac-Man?branch=master
[demo]: https://xevion.github.io/Pac-Man/
[commits]: https://github.com/Xevion/Pac-Man/commits/master

A faithful recreation of the classic Pac-Man arcade game written in Rust. This project aims to replicate the original game's mechanics, graphics, sound, and behavior as accurately as possible while providing modern development features like cross-platform compatibility and WebAssembly support.

The game includes all the original features you'd expect from Pac-Man:

- [x] Classic maze navigation and dot collection
- [ ] Four ghosts with their unique AI behaviors (Blinky, Pinky, Inky, and Clyde)
- [ ] Power pellets that allow Pac-Man to eat ghosts
- [ ] Fruit bonuses that appear periodically
- [ ] Progressive difficulty with faster ghosts and shorter power pellet duration
- [x] Authentic sound effects and sprites

This cross-platform implementation is built with SDL2 for graphics, audio, and input handling. It can run on Windows, Linux, macOS, and in web browsers via WebAssembly.

## Why?

Just because. And because I wanted to learn more about Rust, inter-operability with C, and compiling to WebAssembly.

I was inspired by a certain code review video on YouTube; [SOME UNIQUE C++ CODE // Pacman Clone Code Review](https://www.youtube.com/watch?v=OKs_JewEeOo) by The Cherno.

For some reason, I was inspired to try and replicate it in Rust, and it was uniquely challenging.

I wanted to hit a log of goals and features, making it a 'perfect' project that I could be proud of.

- Near-perfect replication of logic, scoring, graphics, sound, and behaviors. No hacks, workarounds, or poor designs.
- Written in Rust, buildable on Windows, Linux, Mac and WebAssembly. Statically linked, no runtime dependencies.
- Performant, low memory, CPU and GPU usage.
- Online demo, playable in a browser.
- Completely automatic build system with releases for all platforms.
- Well documented, well-tested, and maintainable.

## Experimental Ideas

- Debug tooling
  - Game state visualization
  - Game speed controls + pausing
  - Log tracing
  - Performance details
- Customized Themes & Colors
  - Color-blind friendly
- Perfected Ghost Algorithms
- More than 4 ghosts
- Custom Level Generation
  - Multi-map tunnelling
- Online Scoreboard
  - An online axum server with a simple database and OAuth2 authentication.
  - Integrates with GitHub, Discord, and Google OAuth2 to acquire an email identifier & avatar.
    - Avatars are optional for score submission and can be disabled, instead using a blank avatar.
    - Avatars are downscaled to a low resolution pixellated image to maintain the 8-bit aesthetic.
    - A custom name is used for the score submission, which is checked for potential abusive language.
      - A max length of 14 characters, and a min length of 3 characters.
      - Names are checked for potential abusive language via an external API.
  - The client implementation should require zero configuration, environment variables, or special secrets.
    - It simply defaults to the pacman server API, or can be overriden manually.

## Build Notes

Since this project is still in progress, I'm only going to cover non-obvious build details. By reading the code, build scripts, and copying the online build workflows, you should be able to replicate the build process.

- We use rustc 1.86.0 for the build, due to bulk-memory-opt related issues on wasm32-unknown-emscripten.
  - Technically, we could probably use stable or even nightly on desktop targets, but using different versions for different targets is a pain, mainly because of clippy warnings changing between versions.
- Install `cargo-vcpkg` with `cargo install cargo-vcpkg`, then run `cargo vcpkg build` to build the requisite dependencies via vcpkg.
- For the WASM build, you need to have the Emscripten SDK cloned; you can do so with `git clone https://github.com/emscripten-core/emsdk.git`
  - The first time you clone, you'll need to install the appropriate SDK version with `./emsdk install 3.1.43` and then activate it with `./emsdk activate 3.1.43`. On Windows, use `./emsdk/emsdk.ps1` instead.
    - I'm still not sure _why_ 3.1.43 is required, but it is. Perhaps in the future I will attempt to use a more modern version.
    - Occasionally, the build will fail due to dependencies failing to download. I even have a retry mechanism in the build workflow due to this.
  - You can then activate the Emscripten SDK with `source ./emsdk/emsdk_env.sh` or `./emsdk/emsdk_env.ps1` or `./emsdk/emsdk_env.bat` depending on your OS/terminal.
  - While using the `web.build.ts` is not technically required, it simplifies the build process and is very helpful.
    - It is intended to be run with `bun`, which you can acquire at [bun.sh](https://bun.sh/)
  - Tip: You can launch a fileserver with `python` or `caddy` to serve the files in the `dist` folder.
    - `python3 -m http.server 8080 -d dist`
    - `caddy file-server --root dist` (install with `[sudo apt|brew|choco] install caddy` or [a dozen other ways](https://caddyserver.com/docs/install))
- `web.build.ts` auto installs dependencies, but you may need to pass `-i` or `--install=fallback|force` to install missing packages. My guess is that if you have some packages installed, it won't install any missing ones. If you have no packages installed, it will install all of them.
  - If you want to have TypeScript resolution for development, you can manually install the dependencies with `bun install` in the `assets/site` folder.
