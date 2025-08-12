set shell := ["bash", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

coverage_exclude_pattern := "app.rs|audio.rs"

coverage:
    # Run & generate report
    cargo llvm-cov \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}" \
    --output-path lcov.info \
    --profile coverage \
    --no-fail-fast nextest

    # Display report
    cargo llvm-cov report \
    --ignore-filename-regex "{{ coverage_exclude_pattern }}"
