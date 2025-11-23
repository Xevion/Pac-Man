set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

binary_extension := if os() == "windows" { ".exe" } else { "" }

# Display available recipes
default:
	just --list

# Open HTML coverage report
html: coverage
	cargo llvm-cov report \
	# prevents the absolute path from being used in the generated report
	--remap-path-prefix \
	--html \
	--open

# Display coverage report
report-coverage: coverage
	cargo llvm-cov report --remap-path-prefix

# Generate baseline LCOV report
coverage:
	cargo +nightly llvm-cov \
	--lcov \
	--remap-path-prefix \
	--workspace \
	--output-path lcov.info \
	--profile coverage \
	--no-fail-fast nextest

# Profile the project using samply
samply:
	cargo build --profile profile
	samply record ./target/profile/pacman{{ binary_extension }}

# Build the project for Emscripten
web *args:
	bun run pacman/web.build.ts {{args}}
	bun run --cwd web build
	caddy file-server --root web/dist/client

# Fix linting errors & formatting
fix:
	cargo fix --workspace --lib --allow-dirty
	cargo fmt --all

# Push commits & tags
push:
	git push origin --tags;
	git push

# Create a postgres container for the server
server-postgres:
	bun run .scripts/postgres.ts

# Build the server image
server-image:
	# build the server image
	docker build \
	--platform linux/amd64 \
	--file ./pacman-server/Dockerfile \
	--tag pacman-server \
	.

# Build and run the server in a Docker container
run-server: server-image
	# remove the server container if it exists
	docker rm --force --volumes pacman-server

	# run the server container
	docker run \
	--rm \
	--stop-timeout 2 \
	--name pacman-server \
	--publish 3000:3000 \
	--env PORT=3000 \
	--env-file pacman-server/.env \
	pacman-server
