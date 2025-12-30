set shell := ["bash", "-c"]

mod pacman 'pacman/Justfile'
mod server 'pacman-server/Justfile'
mod web 'web/Justfile'

# Display available recipes
default:
    @just --list --list-submodules

alias b := bun
alias bu := bun

# Runs 'bun' from within the 'web/' folder
bun *args:
	cd web/ && bun {{args}}

alias bx := bunx
alias bux := bunx

# Runs 'bunx' from within the 'web/' folder
bunx *args:
	cd web/ && bunx {{args}}

# Run the game (pacman::run)
run *args:
    @just pacman::run {{args}}

# Run all checks (Rust workspace + web)
check:
    @echo "Checking format..."
    @cargo fmt --all -- --check || echo "⚠ Format issues detected (run \`just format\` to fix)"
    @echo "Running clippy for desktop target..."
    @cargo clippy --workspace --all-targets --all-features --quiet -- -D warnings || true
    @echo "Running clippy for wasm target..."
    @cargo clippy -p pacman --target wasm32-unknown-emscripten --all-targets --all-features --quiet -- -D warnings || true
    @echo "Running web checks..."
    @just web::check || true
    @echo "Check complete!"

alias lint := check

# Run tests (Rust workspace + web)
test:
    cargo nextest run --workspace --no-fail-fast
    @just web::test || true

# Format code (Rust workspace + web)
format:
    cargo fmt --all
    @just web::format

alias fmt := format

# Dev servers (web + server)
dev:
    @just web::dev

# Build and preview frontend (web::up)
up:
    @just web::up

alias vcpkg := pacman::vcpkg
