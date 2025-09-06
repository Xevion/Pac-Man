set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Regex to exclude files from coverage report, double escapes for Justfile + CLI
# You can use src\\\\..., but the filename alone is acceptable too
coverage_exclude_pattern := "src\\\\app\\.rs|audio\\.rs|src\\\\error\\.rs|platform\\\\emscripten\\.rs|bin\\\\.+\\.rs|main\\.rs|platform\\\\desktop\\.rs|platform\\\\tracing_buffer\\.rs|platform\\\\buffered_writer\\.rs|systems\\\\debug\\.rs|systems\\\\profiling\\.rs"

binary_extension := if os() == "windows" { ".exe" } else { "" }

# !!! --ignore-filename-regex should be used on both reports & coverage testing
# !!! --remap-path-prefix prevents the absolute path from being used in the generated report

# Generate HTML report (for humans, source line inspection)
html: coverage-lcov
    cargo llvm-cov report \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --html \
    --open

# Display report (for humans)
report-coverage: coverage-lcov
    cargo llvm-cov report \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}"

# Run & generate LCOV report (as base report)
coverage-lcov:
    cargo llvm-cov \
    --lcov \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --output-path lcov.info \
    --profile coverage \
    --no-fail-fast nextest

# Run & generate Codecov report (for CI)
coverage-codecov:
    cargo llvm-cov \
    --codecov \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --output-path codecov.json \
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
