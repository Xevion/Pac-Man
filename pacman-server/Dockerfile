ARG RUST_VERSION=1.89.0
ARG GIT_COMMIT_SHA

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

# -- Planner stage --
FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin pacman-server --recipe-path recipe.json

# -- Frontend builder stage --
FROM oven/bun:1 AS frontend-builder
WORKDIR /app

# Copy frontend package files first for layer caching
COPY web/package.json web/bun.lock* ./
RUN bun install --frozen-lockfile

# Copy all frontend source including public directory (contains WASM files)
COPY web/ ./

# Build the frontend (Vite will copy public/ contents to dist/client/)
RUN bun run build

# -- Backend builder stage --
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --bin pacman-server --recipe-path recipe.json

# Copy the source code AFTER, so that dependencies are already cached
COPY . .

# Install build dependencies, then build the server
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --package pacman-server --release --bin pacman-server

# -- Runtime stage --
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/pacman-server /usr/local/bin/pacman-server

# Copy frontend static files from frontend-builder stage
COPY --from=frontend-builder /app/dist/client /app/static

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

ARG TZ=Etc/UTC
ENV TZ=${TZ}

# Optional build-time environment variable for embedding the Git commit SHA
ARG GIT_COMMIT_SHA
ENV GIT_COMMIT_SHA=${GIT_COMMIT_SHA}

# Specify PORT at build-time or run-time, default to 3000
ARG PORT=3000
ENV PORT=${PORT}
EXPOSE ${PORT}

# Set static files directory for the server to serve
ENV STATIC_FILES_DIR=/app/static

CMD ["sh", "-c", "exec /usr/local/bin/pacman-server"]
