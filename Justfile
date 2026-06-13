set dotenv-load
set shell := ["bash", "-c"]

mod pacman 'pacman/Justfile'

alias c := check
alias d := dev
alias f := format
alias fmt := format
alias t := test
alias vcpkg := pacman::vcpkg

default:
    @just --list

# Validate all code (format + clippy desktop/wasm + web check/lint) via tempo
check *flags:
    tempo check {{flags}}

# Auto-format all code (Rust workspace + web)
format *flags:
    tempo fmt {{flags}}

# Lint all code (clippy + eslint)
lint *flags:
    tempo lint {{flags}}

# Run the Rust workspace + web test suites
test *flags:
    tempo test {{flags}}

# Full-stack dev: Postgres + server (watched) + Vite frontend
dev *flags:
    tempo dev {{flags}}

# Manage the local Postgres container (start | reset | rm)
db *args:
    tempo db {{args}}

# Run the desktop game (e.g. `just run -r` for release)
run *args:
    cargo run -p pacman {{args}}

# Build the web bundle (WASM game + SvelteKit frontend)
web-build *args:
    bun run pacman/web.build.ts {{args}}
    bun run --cwd web build

# Build the web bundle and serve it with Caddy
serve *args:
    bun run pacman/web.build.ts {{args}}
    bun run --cwd web build
    caddy file-server --root web/dist/client --listen :${PACMAN_WEB_PORT:-42565}

# Build and preview the frontend
up:
    bun run --cwd web build
    bun run --cwd web preview

alias b := bun
alias bu := bun

# Run 'bun' from within the 'web/' folder
bun *args:
    cd web/ && bun {{args}}

alias bx := bunx
alias bux := bunx

# Run 'bunx' from within the 'web/' folder
bunx *args:
    cd web/ && bunx {{args}}
