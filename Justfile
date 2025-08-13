set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

coverage_exclude_pattern := "app.rs|audio.rs|error.rs"

# Display report (for humans)
report-coverage: coverage
    cargo llvm-cov report \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}"

# Run & generate report (for CI)
coverage:
    cargo llvm-cov \
    --lcov \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --output-path lcov.info \
    --profile coverage \
    --no-fail-fast nextest
