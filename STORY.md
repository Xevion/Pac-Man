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

## Emscripten v.s. SDL2-TTF

While working on the next extension of SDL2 for my test repository, SDL2-TTF had some pretty annoying issues. It would build fine, but it would raise a runtime error: `indirect call to null`.

Luckily, I had a recently updated repository to copy off of, and the working fix was to lower the EMSDK version to `3.1.43`.

[Source](https://github.com/aelred/tetris/blob/0ad88153db1ca7962b42277504c0f7f9f3c675a9/tetris-sdl/src/main.rs#L34)
```rust
static FONT_DATA: &[u8] = include_bytes!("../assets/TerminalVector.ttf");

#[cfg(not(target_os = "emscripten"))]
fn ttf_context() -> ttf::Sdl2TtfContext {
    ttf::init().unwrap()
}

#[cfg(target_os = "emscripten")]
fn ttf_context() -> &'static ttf::Sdl2TtfContext {
    // Deliberately leak so we get a static lifetime
    Box::leak(Box::new(ttf::init().unwrap()))
}

const FONT_MULTIPLE: u16 = 9;

// Funny division is done here to round to nearest multiple of FONT_MULTIPLE
const FONT_SIZE: u16 = (WINDOW_HEIGHT / 32) as u16 / FONT_MULTIPLE * FONT_MULTIPLE;

fn main() {
  ...

  let font_data = RWops::from_bytes(FONT_DATA).unwrap();
  let font_size = max(FONT_SIZE, FONT_MULTIPLE);
  let font = ttf_context
      .load_font_from_rwops(font_data, font_size)
      .unwrap();
}
```

I don't particularly understand why loading from memory is used, but it's a neat trick. I tested normal font loading afterwards, and it seems to be totally fine.

On to the Mixer extension, then.

## Mixer and GFX

Mixer was relatively easy, I don't remember anything special about it.

As it happens, neither was SDL GFX, except for me finding that getting it compiling on Windows would soon be difficult; `SDL2_gfx` is not currently being updated, nor is it managed by the SDL team. This meant that no releases of development libraries including DLLs or LIB files were going to be available.

When I added in GFX, I wanted to add some mouse interaction since that currently wasn't being done anywhere in the demo, but I also wanted the ability for the mouse to be hidden until used.

Detecting whether the mouse was focusing the window or not wasn't super easy, and I'm still not sure that it's working perfectly, but at the very least Emscripten seems to support what I'm trying to do. I should look into asynchronous Javascript callbacks, see what Emscripten supports.

## Styling with PostCSS + Tailwind

I'm big on using Tailwind, and while this project probably could have done without it, I didn't want to forego my favorite tool.

But I also didn't want to include some big framework on this, like Astro, so I looked for the smallest way to include Tailwind.

After fiddling and failing to find Hugo suitable, I stuck to plain HTML & the PostCSS method, which worked great. It's definitely not that fast for rapid development, but it works well enough.

The only thing I'm unsatisfied with is why `postcss-cli` wasn't working when executed from `pnpm`. It works just fine from `pnpx`, but it has to download and setup the whole package on *every single invocation*, which is super slow. And probably expensive, in the long run.

## Cross-platform Builds

With the next step of the demo project, I needed to get builds for every OS running, that's one down out of the four targets I'm gunning for.

Linux was the easiest, as usual, with `apt` providing access to all the development libraries of SDL & the associated extensions, including `SDL2_gfx`.

There's also no requirement for providing sidecar DLLs like Windows needs, so that worked well.
The hardest part was figuring out the most satisfying way to zip and load all the assets together, but luckily the artifact uploader provides it's own zip implementation; albeit I may need to modify it to add further system hinting (`.tar.gz` for Linux, `.dmg` for MacOS, `.zip` for Windows).

## SDL2 on Windows

SDL2 on Windows has to be one of the least fun development cycles; setting up the environment is pretty painful as there's almost no guides for Rust users to figure out each requirement. You'll learn fast, and this knowledge is hands on experience that will probably be applicable later on in C++ development, but I'm sure a fair number of Rust users like myself have no idea why a DLL or LIB file is necessary at all.

To be honest, I still don't.

Regardless, SDL2 needs a LIB file for compliation to be available in the root directory, and each extension has there own.

Once the EXE is compiled, the working directory needs to contain a DLL file for execution, too. Each extension has it's own as well.

This sounds easy, but acquiring these DLLs and LIB files is not easy. At the very least, the SDL-supported extensions have releases available containing

![SDL2 Mixer GitHub Release Files](https://i.xevion.dev/ShareX/2024/04/firefox_6zAmbsD97n.png)

So I got to creating a build step involving the download of each of these libraries. I'm no expert with `curl`, but I had it figured out eventually.

```yaml
- name: Download SDL2 Libraries
  run: |
    curl -L "https://github.com/libsdl-org/SDL/releases/download/release-${{ env.SDL2 }}/SDL2-devel-${{ env.SDL2 }}-VC.zip" -o "sdl2_devel.zip"
    curl -L "https://github.com/libsdl-org/SDL_mixer/releases/download/release-${{ env.SDL2_MIXER }}/SDL2_mixer-devel-${{ env.SDL2_MIXER }}-VC.zip" -o "sdl2_mixer_devel.zip"
    curl -L "https://github.com/libsdl-org/SDL_ttf/releases/download/release-${{ env.SDL2_TTF }}/SDL2_ttf-devel-${{ env.SDL2_TTF }}-VC.zip" -o "sdl2_ttf_devel.zip"
    curl -L "https://github.com/libsdl-org/SDL_image/releases/download/release-${{ env.SDL2_IMAGE }}/SDL2_image-devel-${{ env.SDL2_IMAGE }}-VC.zip" -o "sdl2_image_devel.zip"
```

I did take a lot of care in making sure that versions were specified externally in different variables, which took a couple tries while I learned how interpolation works with GitHub Actions.

Additionally, I realized that `LIB` files were required for compliation after this, so I had to painfully fix all the files to use the `-devel-` version. Speifically the one with `-VC` appended.

I still do not know what VC means here. Perhaps it is related to `vcpkg` somehow.

The next step was to extract the files I needed from the `.zip`s, but that proved quite hard. I'm a lover of precision and using tools to the best of my knowledge, so I wanted to finely take just the DLL and ZIP I needed from these archives, and nothing else.

While I was able to get working commands to do this on Linux, finely finding the exact DLL and placing it in `pwd`, I was not able to replicate it on the Windows-imaged GitHub Runner;

When specifying the `-o` flag meaning 'output directory here' like `-o./tmp` (yes, there is no space in between), it would always error with `Too short switch: -o`. I was unable to find meaningful discussions on Google.

My Linux machine did not complain, and I wasn't yet ready to switch OSes for an error like this, so I just extracted everything and then `mv`'d the items into `pwd`.

I knew what lay ahead with `SDL2_gfx`, so I tested whether the compilation error changed, and luckily, it was only erroring on the missing `SDL2_gfx.lib` at this point.

While reading discussions online, I came across [a reddit post](https://www.reddit.com/r/rust_gamedev/comments/am84q9/using_sdl2_gfx_on_windows/efk6uwq/) talking about `vcpkg`. I'd heard of it, but never used the program before. It seemed like it could provide `SDL2_gfx` for me without hassle.

And that was partly true.

The primary 'boon' of `vcpkg` here was that it setup and compiled `SDL2_gfx` without the hassle of messing with the compiler, options, or most importantly: dependencies.

I didn't know it at the time, but `SDL2_gfx` depended on `SDL2` directly, and so I'd have to setup and compile both projects, if I was hoping to do this 'manually'.

## VCPKG for SDL2_GFX

I tried to use the GitHub-provided environment variables relating to VCPKG's installation location, but nothing really worked here. I was on the correcti mage (`windows-latest` for Windows 2022 Enterprise on GitHub's Runner Images), but nothing seemed to work.

[This comment](https://github.com/actions/runner-images/issues/6376#issuecomment-1781269302) seemed to describe the exact same experience I was happening, several months ago.

Alas, I simply tried `C:\vcpkg\` and it worked, providing me the ability to install `SDL2_gfx`.

As it were though, the hard part wasn't going to be compiling, but locating the DLLs and LIB files for movement. No matter where I looked online or in the logs, nothing was obvious about the location of my files.

In retrospect, a recursive `Get-ChildItem` looking for `DLL` or `LIB` files probably would've worked well, but.. yeah...

After a couple attempts with various test commits, I couldn't find it, and just switched to Windows to install and compile it myself, so I could locate the file manually.

> Note: VCPKG is annoying to install, the executable provided by Visual Studio Community does not permit classic-mode usage, so you'll still need to clone and bootstrap VCPKG (instructions in the repository README).

As it happens, they were placed in
- `$VCPKG_ROOT\packages\sdl2-gfx_x64-windows-release\bin\SDL2_gfx.dll` and
- `$VCPKG_ROOT\packages\sdl2-gfx_x64-windows-release\lib\SDL2_gfx.lib` respectively.

This brings me to one issue, and one fix; while compiling you're required to specify that the build is for 64 bit systems manually, on each invocation of VCPKG (while in classic mode, which I am).

On top of that, they'll be built in debug mode (with extra symbols and such) by default, which I am not interested in.

To get the x64 Release build of a package, append `:x64-windows-release` to it, as in `sdl2-gfx:x64-windows-release` for `sdl2-gfx`.

After getting this sorted, I struggled a little bit in using the `mv` (`Move-Item`) command in Powershell, as I battled with the comma delimited files when moving multiple files to a given destination. Dumb.

This is also the point at which I renamed the executable from `pacman` to `spiritus` to differentiate the two projects. The name is just my play on the word 'sprite'.

## Console Window Hiding

When launching the demo app, I saw a console window pop up, even though I launched it from the File Explorer; this is not the behavior I was interested in.

I believe that apps launched from File Explorer shouldn't have a console window available unless...

- It's a CLI app by nature, and it uses the Console Window.
- It has a specific debugging flag passed into it, perhaps by a Shortcut file.
- The console window is required for the nature of the app, or it is the preferred method of log inspection.
- It's a debug build.

But if it's launched from the console, then it should either

- Detach and relinquish control of the console back to the user.
- Use the console actively in it's logging.

Most programs I know and use follow this general consensus. Naturally, mine must too.

But, when searching for a solution online, it seemed what I want doesn't really exist; I implemented the closest approximation.

If `stdout` is detected to be a `tty` (an active console), the console window won't be hidden. Otherwise, it will be hidden.

Unfortunately, this results in the millisecond flash of a black console window appearing.

## Updating Deprecated Actions

As it were, most of the actions I were using were deprecated in some way. It didn't feel like I was using super old actions, but I guess I was. Luckily, most of them were simply just updating the version (`2` or `3` to `4` or `5`).

`actions-rs@toolchain` was different though, and was officially deprecated, the GitHub repository archived. Couldn't find a good reason why, but the repository was untouched in 4 years, so maybe that's why...

I found `dtolnay@rust-toolchain` and switched, it more or less was perfect with no differences. I think it's sorta neat that the Rust version is specified using the version of the action. I'd be worried though of a changing feature set across different action versions...

I guess a well designed GitHub Action shouldn't change much, including a Rust toolchain action.

## Artifact Naming

Perhaps it's super unnecessary and won't be appreciated, but I wanted the artifact files produced by my script to have semantic meaning in it's version and target.

For each OS, I extracted the targets (`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`...) into an environment variable scoped at the job level (love that).

I looked into ways to get the package version, but nothing obvious jumped out at me. I did come across `toml-cli` though, a Rust-based CLI program analogous to `jq` (for JSON).

It worked great, but it was sorta slow; compiling `toml-cli` added an extra 20-60 seconds for each job. I'd heard of a Rust project to speed up builds by providing prebuilt executables though; it's called `binstall`.

Even cooler, it was had an action available to easily add it to my build script, and so I had `toml-cli` installing and available 10x faster!

![GitHub Actions output showing 2 cargo binstall process taking 4 seconds total](https://i.xevion.dev/ShareX/2024/04/firefox_5sOEDM7zkc.png)

Perhaps I could use some special bash commands to acquire the package version, but it'd be a lot of work and maintenance to get it working in both bash and Powershell, maintaining it across four jobs.

This is both cool, fast, and easy!

![GitHub Artifacts with Sizes Listed](https://i.xevion.dev/ShareX/2024/04/firefox_jDI8MDwWRc.png)

I was thinking of a github-pages artifact name that aligns with the others, but I think that'd be stupid AND overkill.

Perhaps at the least I'll look into a 32-bit build for Windows, just for demonstration purposes.


[code-review-video]: https://www.youtube.com/watch?v=OKs_JewEeOo
[code-review-thumbnail]: https://img.youtube.com/vi/OKs_JewEeOo/hqdefault.jpg
[fighting-lifetimes-1]: https://devcry.heiho.net/html/2022/20220709-rust-and-sdl2-fighting-with-lifetimes.html
[fighting-lifetimes-2]: https://devcry.heiho.net/html/2022/20220716-rust-and-sdl2-fighting-with-lifetimes-2.html
[fighting-lifetimes-3]: https://devcry.heiho.net/html/2022/20220724-rust-and-sdl2-fighting-with-lifetimes-3.html
[ruggrogue]: https://tung.github.io/ruggrogue/