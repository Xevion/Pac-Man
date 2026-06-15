//! Tracy live-profiler integration, gated behind the `tracy` feature.
//!
//! Everything here compiles to a no-op when `tracy` is disabled, so call sites
//! stay unconditional. With the feature on, the client is started during console
//! init and a [`TracyLayer`](tracing_tracy::TracyLayer) is added to the tracing
//! subscriber so every `tracing` span becomes a Tracy zone. Per-system zones
//! (named at runtime) and plots are emitted directly through `tracy-client`,
//! bypassing the compile-time `release_max_level_debug` filter that would
//! otherwise strip trace-level spans from profiling builds.
//!
//! Connect a Tracy GUI to `127.0.0.1` while a `tracy`-enabled binary runs.

// The Tracy crates are desktop-only dependencies, so the feature can only be
// honored off the Emscripten target. `--all-features` still flips the flag on
// wasm builds, hence the `not(target_os = "emscripten")` guard on every site.
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
pub use tracy_client as client;

/// RAII guard that ends its Tracy zone when dropped. Bind it to a named local
/// (`let _zone = ...`) for the lifetime of the region you want measured.
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
pub struct ZoneGuard(#[allow(dead_code)] Option<tracy_client::Span>);

#[cfg(not(all(feature = "tracy", not(target_os = "emscripten"))))]
pub struct ZoneGuard;

/// Open a Tracy zone with a runtime-chosen name. The allocation happens inside
/// the Tracy client, so this is cheap enough for per-system, per-frame use but
/// not for inner-loop leaves.
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
#[inline]
pub fn zone(name: &str, file: &str, line: u32) -> ZoneGuard {
    ZoneGuard(tracy_client::Client::running().map(|c| c.span_alloc(Some(name), "", file, line, 0)))
}

#[cfg(not(all(feature = "tracy", not(target_os = "emscripten"))))]
#[inline]
pub fn zone(_name: &str, _file: &str, _line: u32) -> ZoneGuard {
    ZoneGuard
}

/// Mark the boundary between frames. Tracy uses these to compute per-frame
/// timing and to drive its "show only slow frames" workflow.
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
#[inline]
pub fn frame_mark() {
    tracy_client::frame_mark();
}

#[cfg(not(all(feature = "tracy", not(target_os = "emscripten"))))]
#[inline]
pub fn frame_mark() {}

/// Annotate the Tracy timeline with a one-off message (e.g. a window resize).
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
#[inline]
pub fn message(text: &str) {
    if let Some(c) = tracy_client::Client::running() {
        c.message(text, 0);
    }
}

#[cfg(not(all(feature = "tracy", not(target_os = "emscripten"))))]
#[inline]
pub fn message(_text: &str) {}

/// Build the Tracy tracing layer and start the client. Called once from the
/// platform console init; the returned layer captures every `tracing` span.
#[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
pub fn layer() -> Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync> {
    use tracing_subscriber::fmt::format::DefaultFields;
    use tracing_subscriber::Layer;

    struct Config {
        fmt: DefaultFields,
    }

    impl tracing_tracy::Config for Config {
        type Formatter = DefaultFields;

        fn formatter(&self) -> &Self::Formatter {
            &self.fmt
        }

        // Keep span fields out of the zone name so spans collapse into one
        // zone per name in Tracy's Statistics view.
        fn format_fields_in_zone_name(&self) -> bool {
            false
        }
    }

    tracy_client::Client::start();
    tracing_tracy::TracyLayer::new(Config {
        fmt: DefaultFields::default(),
    })
    .boxed()
}

/// Emit a Tracy plot sample under a static plot name. No-op without the feature.
#[macro_export]
macro_rules! tracy_plot {
    ($name:literal, $value:expr) => {{
        #[cfg(all(feature = "tracy", not(target_os = "emscripten")))]
        {
            $crate::tracy::client::plot!($name, $value);
        }
        #[cfg(not(all(feature = "tracy", not(target_os = "emscripten"))))]
        {
            let _ = $value;
        }
    }};
}
