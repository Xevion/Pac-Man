ARG RUST_VERSION=1.86.0
ARG EMSDK_VERSION=4.0.22
ARG GIT_COMMIT_SHA

# ========== Stage 1: WASM Base (shared toolchain) ==========
# This stage installs Rust and cargo-chef once, then is reused by other WASM stages
FROM emscripten/emsdk:${EMSDK_VERSION} AS wasm-base
ARG RUST_VERSION
WORKDIR /app

# Install Rust with WASM target and cargo-chef (cached layer)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain ${RUST_VERSION} \
    --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add wasm32-unknown-emscripten && \
    cargo install cargo-chef --locked

# ========== Stage 2: WASM Planner ==========
FROM wasm-base AS wasm-planner

# Copy workspace for recipe generation (minimal files needed)
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo/ ./.cargo/
COPY pacman-common/ ./pacman-common/
COPY pacman/ ./pacman/
COPY pacman-server/ ./pacman-server/

# Generate dependency recipe
RUN cargo chef prepare --bin pacman --recipe-path recipe.json

# ========== Stage 3: WASM Dependency Builder ==========
FROM wasm-base AS wasm-deps

# Cook dependencies only (this layer is cached until Cargo.toml/Cargo.lock change)
# Note: Assets are required during linking due to --preload-file in rustflags
COPY --from=wasm-planner /app/recipe.json recipe.json
COPY .cargo/ ./.cargo/
COPY pacman/assets/game/ ./pacman/assets/game/
RUN cargo chef cook --release --target wasm32-unknown-emscripten --recipe-path recipe.json

# ========== Stage 4: WASM Builder ==========
FROM wasm-base AS wasm-builder

# Copy cached dependencies from wasm-deps
COPY --from=wasm-deps /app/target target
COPY --from=wasm-deps /root/.cargo /root/.cargo

# Copy workspace source (build from workspace root so preload-file paths work)
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY .cargo/ ./.cargo/
COPY pacman-common/ ./pacman-common/
COPY pacman/ ./pacman/
COPY pacman-server/ ./pacman-server/

# Build WASM binary (dependencies already cached)
RUN cargo build --release --target wasm32-unknown-emscripten --bin pacman

# ========== Stage 5: Frontend Builder ==========
FROM oven/bun:1 AS frontend-builder
WORKDIR /app

# Copy package files for dependency installation (cached until package.json/bun.lock change)
COPY web/package.json web/bun.lock* ./
RUN bun install --frozen-lockfile

# Copy frontend source first (so we have the static/ directory)
COPY web/ ./

# Copy WASM artifacts from wasm-builder stage to SvelteKit's static folder
# Note: .wasm and .js are in release/, but .data (preloaded assets) is in release/deps/
COPY --from=wasm-builder /app/target/wasm32-unknown-emscripten/release/pacman.wasm ./static/pacman.wasm
COPY --from=wasm-builder /app/target/wasm32-unknown-emscripten/release/pacman.js ./static/pacman.js
COPY --from=wasm-builder /app/target/wasm32-unknown-emscripten/release/deps/pacman.data ./static/pacman.data

# Verify WASM artifacts exist and have reasonable sizes
RUN test -f ./static/pacman.wasm && \
    test -f ./static/pacman.js && \
    test -f ./static/pacman.data && \
    [ $(stat -c%s ./static/pacman.wasm) -gt $((1024 * 1024)) ] && \
    [ $(stat -c%s ./static/pacman.js) -gt $((100 * 1024)) ] && \
    [ $(stat -c%s ./static/pacman.data) -gt $((10 * 1024)) ] && \
    echo "WASM artifacts verified (wasm >1MiB, js >100KiB, data >10KiB)" || \
    (echo "WASM artifacts missing or too small!" && exit 1)

# Build frontend (SvelteKit bundles WASM files from static/)
RUN bun run build

# ========== Stage 6: Backend Chef ==========
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

# Install build dependencies FIRST (cached layer - rarely changes)
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# ========== Stage 7: Backend Planner ==========
FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin pacman-server --recipe-path recipe.json

# ========== Stage 8: Backend Builder ==========
FROM chef AS builder

# Cook dependencies first (cached until Cargo.toml/Cargo.lock change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin pacman-server --recipe-path recipe.json

# Copy source code and build (only this layer invalidates on code changes)
COPY . .
RUN cargo build --package pacman-server --release --bin pacman-server

# ========== Stage 9: Runtime ==========
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies (curl needed for healthcheck)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tzdata curl && \
    rm -rf /var/lib/apt/lists/*

# Copy compiled server binary
COPY --from=builder /app/target/release/pacman-server /usr/local/bin/pacman-server

# Copy frontend static files (includes WASM)
COPY --from=frontend-builder /app/dist/client /app/static

# Environment configuration
ARG TZ=Etc/UTC
ENV TZ=${TZ}

ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

ARG PORT=3000
ENV PORT=${PORT}
EXPOSE ${PORT}

ENV STATIC_FILES_DIR=/app/static

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:${PORT}/api/health || exit 1

CMD ["pacman-server"]
