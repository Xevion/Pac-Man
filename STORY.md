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

## Key Capturing Extensions in WASM Build

Some extensions I had installed were capturing keys.
The issue presented with some keys never being sent to the application.
To confirm, enter safe mode or switch to a different browser without said extensions.
If the issue disappears, it's because of an extension in your browser stealing keys in a way that is incompatible with the batshit insanity of Emscripten.


## A Long Break

After hitting a wall with an issue with Emscripten where the tab would freeze after switching tabs (making it into a background tab), I decided to take a break from the project. A couple months went by without anything going on.

## Revisiting

I decided to revisit the project because I didn't want to see this project die. It's actually a lot of fun, and has a very interesting stack, with a simple premise, and a lot of potential for expansion.

Unfortunately, the issue above still lingered. I did a lot of testing, and concluded that I needed to create a simple example with as much stripped away as possible. All I learned from this was that the freeze occurred the moment that the 'Hidden' event (for the Window) was fired. After that, the rendered would take 0 nanoseconds to render, and some script for Asyncify would keep spinning in the background.

I tried to ask around but didn't get anywhere, but one reply on my post gave me the idea to back away from Emscripten 1.39.20 (several years old at this point).

## Emscripten Callback Main Loop

I looked into as many examples online (not that many), and came across an Emscripten callback loop exposed in C. Some were basic and all over the place, some were advanced, but also imbibed extremely annoying static lifetime requirements.

- I tried my best to satisfy and work with these lifetimes, but it was a nightmare.
- Instead, I tried to simplify and move away from this annoying Emscripten callback loop, but simpler ones had issues, crashing with `invalid renderer` errors.
  - This guy named Greg Buchholz apparently was the creator of this special Emscripten bindings with the static lifetimes, and it was done to solve this issue with `invalid renderer`.
  - [GitHub](https://github.com/gregbuchholz), [Repository](https://github.com/gregbuchholz/RuSDLem), [StackOverflow](https://stackoverflow.com/questions/69748049/rust-sdl2-emscripten-and-invalid-renderer-panic), [Forum Post](https://users.rust-lang.org/t/sdl2-emscripten-asmjs-and-invalid-renderer-panic/66567/2)

With this in mind, it seemed like I was at a dead end AGAIN; either I had to deal with the static lifetimes (I am not that good at Rust), or I had to deal with Asyncify.

But this did help me narrow my search even more for a good example. I needed to find a repository with Rust, SDL2, Emscripten, and `TextureCreator`.

`TextureCreator` was key, as the static lifetimes issue was most encumbering when dealing with borrows and lifetimes of `TextureCreator` inside the `main` loop closure.

## Return to Asyncify

I found [one such repository](https://github.com/KyleMiles/Rust-SDL-Emscripten-Template/), and interestingly, it used `latest` Emscripten (not a specific target like 1.39.20), and was new enough (2 years old, but still new enough) to be relevant.

Even more interesting, it didn't use the `main` loop closure, but instead used Emscripten's *Asyncify* feature to handle the main loop.

But, unlike my original project which called `std::thread::sleep` directly, it used bindings into Emscripten's functions like `emscripten_sleep`.

Even better, it had an example of script execution (JavaScript) bindings, which I could use to handle all sorts of things. I tested it out, and it worked.

## Instant::now() 32-bit Byte Cutoff

Unfortunately while trying to get basic FPS timings working, I got divide by zero errors when trying to calculate the time difference between two `Instant` times.

This was weird, and honestly, I'm confused as to why the 2-year old sample code 'worked' at the time, but not now.

After a bit of time, I noted that the `Instant` times were printing with only the whole seconds changing, and the nanoseconds were always 0.

```
Instant { tv_sec: 0, tv_nsec: 0 }
Instant { tv_sec: 1, tv_nsec: 0 }
Instant { tv_sec: 2, tv_nsec: 0 }
Instant { tv_sec: 3, tv_nsec: 0 }
Instant { tv_sec: 4, tv_nsec: 0 }
...
```

This was super weird, but I stumbled upon [an issue on GitHub](https://github.com/rust-lang/rust/issues/113852) that mentioned the exact situation I was in, as well as providing a patch solution (`emscripten_get_now`).

## VSync Gotcha

After getting the timing working, I noticed that the rendering was extremely slow. I was getting 60 FPS, but I wasn't sleeping at all.

Normally when rendering occurs, you want to sleep for the remaining time so that your game calculations can occur at a consistent rate (60 FPS for example).

If your rendering time is less than the sleep time, you can just sleep for the remaining time. But if your rendering time is greater than the sleep time, you encounter lag, the FPS starts to drop.

This was a confusing issue as I knew it couldn't be a coincidence that the rendering time was exactly ~16ms (60 FPS) every time.

After a little bit though, I found the `present_vsync` function in the SDL2 render initialization. This was causing the rendering to try and time the canvas present() to the monitor's refresh rate (60 FPS).

Maybe I could have skipped my custom timing and just used this, but I don't know if it would be platform-independent, what would happen on 120 FPS displays, etc.

[code-review-video]: https://www.youtube.com/watch?v=OKs_JewEeOo
[code-review-thumbnail]: https://img.youtube.com/vi/OKs_JewEeOo/hqdefault.jpg
[fighting-lifetimes-1]: https://devcry.heiho.net/html/2022/20220709-rust-and-sdl2-fighting-with-lifetimes.html
[fighting-lifetimes-2]: https://devcry.heiho.net/html/2022/20220716-rust-and-sdl2-fighting-with-lifetimes-2.html
[fighting-lifetimes-3]: https://devcry.heiho.net/html/2022/20220724-rust-and-sdl2-fighting-with-lifetimes-3.html
[ruggrogue]: https://tung.github.io/ruggrogue/