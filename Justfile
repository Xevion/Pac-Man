set shell := ["bash", "-c"]

mod pacman 'pacman/Justfile'
mod server 'pacman-server/Justfile'
mod web 'web/Justfile'

# Display available recipes
default:
    @just --list --list-submodules

# Run the game (pacman::run)
run *args:
    @just pacman::run {{args}}

# Run all checks (pacman::check)
check:
    @just pacman::check

alias lint := check

# Run tests (pacman::test)
test:
    @just pacman::test

# Format code (pacman::format)
format:
    @just pacman::format

alias fmt := format

# Frontend dev server (web::dev)
dev:
    @just web::dev

# Build and preview frontend (web::up)
up:
    @just web::up

alias vcpkg := pacman::vcpkg
