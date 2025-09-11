set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]


binary_extension := if os() == "windows" { ".exe" } else { "" }

# !!! --ignore-filename-regex should be used on both reports & coverage testing
# !!! --remap-path-prefix prevents the absolute path from being used in the generated report

# Generate HTML report (for humans, source line inspection)
html: coverage
    cargo llvm-cov report \
    --remap-path-prefix \
    --html \
    --open

# Display report (for humans)
report-coverage: coverage
    cargo llvm-cov report --remap-path-prefix

# Run & generate LCOV report (as base report)
coverage:
    cargo +nightly llvm-cov \
    --lcov \
    --remap-path-prefix \
    --workspace \
    --output-path lcov.info \
    --profile coverage \
    --no-fail-fast nextest

# Profile the project using 'samply'
samply:
    cargo build --profile profile
    samply record ./target/profile/pacman{{ binary_extension }}

# Build the project for Emscripten
web *args:
    bun run web.build.ts {{args}};
    caddy file-server --root dist

# Run cargo fix
fix:
    cargo fix --workspace --lib --allow-dirty
    cargo fmt --all
