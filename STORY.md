# Story

This is living document that describes the story of the project, from inspiration to solution.
When a website is available, this document will help curate it's content.

## Inspiration

I initially got the idea for this project after finding a video about another Pac-Man clone on YouTube.

[![Code Review Thumbnail][code-review-thumbnail]][code-review-video]

This implementation was written in C++, used SDL2 for graphics, and was kinda weird - but it worked.

- I think it was weird because the way it linked files together is extremely non-standard.
  Essentially, it was a single file that included all the other files. This is not how C++ projects are typically structured.
- This implementation was also extremely dependent on OOP; Rust has no real counterpart for OOP code, so writing my own implementation would be a challenge.

## Lifetimes

Rust's SDL2 implementation is a wrapper around the C library, so it's not as nice as the C++ implementation.
Additionally, lifetimes in this library are a bit weird, making them quite difficult to deal with.

I found a whole blog post complaining about this ([1][fighting-lifetimes-1], [2][fighting-lifetimes-2], [3][fighting-lifetimes-3]), so I'm not alone in this.

## Emscripten & RuggRogue

One of the targets for this project is to build a web-accessible version of the game. If you were watching at all during
the Rust hype, one of it's primary selling points was a growing community of Rust-based web applications, thanks to
WebAssembly.

The problem is that much of this work was done for pure-Rust applications - and SDL is C++.
This requires a C++ WebAssembly compiler such as Emscripten; and it's a pain to get working.

Luckily though, someone else has done this before, and they fully documented it - [RuggRouge][ruggrouge].
- Built with Rust
- Uses SDL2
- Compiling for WebAssembly with Emscripten
- Also compiles for Windows & Linux

This repository has been massively helpful in getting my WebAssembly builds working.

[code-review-video]: https://www.youtube.com/watch?v=OKs_JewEeOo
[code-review-thumbnail]: https://img.youtube.com/vi/OKs_JewEeOo/hqdefault.jpg
[fighting-lifetimes-1]: https://devcry.heiho.net/html/2022/20220709-rust-and-sdl2-fighting-with-lifetimes.html
[fighting-lifetimes-2]: https://devcry.heiho.net/html/2022/20220716-rust-and-sdl2-fighting-with-lifetimes-2.html
[fighting-lifetimes-3]: https://devcry.heiho.net/html/2022/20220724-rust-and-sdl2-fighting-with-lifetimes-3.html
[ruggrogue]: https://tung.github.io/ruggrogue/