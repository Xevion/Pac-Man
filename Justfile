set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

binary_extension := if os() == "windows" { ".exe" } else { "" }

# Display available recipes
default:
	just --list

# === Quality & Validation ===

# Run all checks (format, lint, test)
check:
	@echo "Running all checks..."
	@just format-check
	@just lint
	@just test
	@echo "All checks passed!"

# Quick format + lint for fast iteration
quick:
	cargo fmt --all
	@cargo clippy --all-targets --all-features --quiet -- -D warnings

# Full CI pipeline
ci: format-check lint test coverage
	@echo "CI pipeline complete!"

# Auto-format code
format:
	cargo fmt --all

# Check formatting without modifying files
format-check:
	cargo fmt --all -- --check

# Run strict multi-platform lints (desktop + wasm)
lint:
	@echo "Running clippy for desktop target..."
	@cargo clippy --all-targets --all-features --quiet -- -D warnings
	@echo "Running clippy for wasm target..."
	@cargo clippy -p pacman --target wasm32-unknown-emscripten --all-targets --all-features --quiet -- -D warnings
	@echo "All lints passed!"

# Fix linting errors & formatting
fix:
	cargo fix --workspace --lib --allow-dirty
	cargo fmt --all

# Security audit for vulnerabilities
audit:
	cargo audit

# Verify required tools are installed
smoke:
	@command -v cargo >/dev/null || (echo "❌ cargo not found" && exit 1)
	@command -v samply >/dev/null || (echo "❌ samply not found" && exit 1)
	@command -v bun >/dev/null || (echo "❌ bun not found" && exit 1)
	@command -v caddy >/dev/null || (echo "❌ caddy not found" && exit 1)
	@echo "✓ All required tools present!"

# === Testing & Coverage ===

# Run tests with nextest
test:
	cargo nextest run --no-fail-fast

# Generate baseline LCOV report
coverage:
	cargo +nightly llvm-cov \
	--lcov \
	--remap-path-prefix \
	--workspace \
	--output-path lcov.info \
	--profile coverage \
	--no-fail-fast nextest

# Display coverage report
report-coverage: coverage
	cargo llvm-cov report --remap-path-prefix

# Open HTML coverage report
html: coverage
	cargo llvm-cov report \
	--remap-path-prefix \
	--html \
	--open

# === Performance & Profiling ===

# Profile the project using samply
samply:
	cargo build --profile profile
	samply record ./target/profile/pacman{{ binary_extension }}

# === Web Build (Emscripten) ===

# Build and serve the web version
web *args:
	bun run pacman/web.build.ts {{args}}
	bun run --cwd web build
	caddy file-server --root web/dist/client --listen :8547

# Build web version only (no server)
build-web *args:
	bun run pacman/web.build.ts {{args}}
	bun run --cwd web build

# === Server (Docker) ===

# Create a postgres container for the server
server-postgres:
	bun run pacman-server/scripts/postgres.ts

# Build the server image
server-image:
	docker build \
	--platform linux/amd64 \
	--file ./pacman-server/Dockerfile \
	--tag pacman-server \
	.

# Build and run the server in a Docker container
run-server: server-image
	docker rm --force --volumes pacman-server 2>/dev/null || true
	docker run \
	--rm \
	--stop-timeout 2 \
	--name pacman-server \
	--publish 3000:3000 \
	--env PORT=3000 \
	--env-file pacman-server/.env \
	pacman-server

# === Utilities ===

# Clean build artifacts
clean:
	cargo clean
