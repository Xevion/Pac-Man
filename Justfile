set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Regex to exclude files from coverage report, double escapes for Justfile + CLI
# You can use src\\\\..., but the filename alone is acceptable too
coverage_exclude_pattern := "src\\\\app.rs|audio.rs|src\\\\error.rs|platform\\\\emscripten.rs"

# !!! --ignore-filename-regex should be used on both reports & coverage testing
# !!! --remap-path-prefix prevents the absolute path from being used in the generated report

# Generate HTML report (for humans, source line inspection)
html: coverage
    cargo llvm-cov report \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --html \
    --open

# Display report (for humans)
report-coverage: coverage
    cargo llvm-cov report \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}"

# Run & generate report (for CI)
coverage:
    cargo llvm-cov \
    --lcov \
    --remap-path-prefix \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --output-path lcov.info \
    --profile coverage \
    --no-fail-fast nextest
