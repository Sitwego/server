#!/bin/bash
set -o pipefail

SILENT=false
PACKAGES=()

if [ "$1" = "--silent" ]; then
    SILENT=true
    shift
fi

log() {
    if [ "$SILENT" = false ]; then
        echo "$@"
    fi
}

# Run a command; in silent mode, capture output and only show it on failure.
run() {
    if [ "$SILENT" = true ]; then
        local output
        if ! output=$("$@" 2>&1); then
            echo "$output" >&2
            return 1
        fi
    else
        "$@"
    fi
}

if [ $# -gt 0 ]; then
    log "Linting specific packages: $*"

    for pkg in "$@"; do
        if ! cargo pkgid --package "$pkg" > /dev/null 2>&1; then
            log "Error: Package '$pkg' not found in workspace"
            exit 1
        fi
        PACKAGES+=(--package "$pkg")
    done

    log "Running rustfmt..."
    if ! run cargo fmt "${PACKAGES[@]}" -- --check; then
        log "Code formatting failed!"
        exit 1
    fi

    log "Running Clippy..."
    if ! run cargo clippy "${PACKAGES[@]}" --all-targets --all-features --no-deps -- -D warnings; then
        log "Linting failed!"
        exit 1
    fi

    log "Code is clean and formatted!"
else
    log "Linting all crates in workspace"

    log "Running rustfmt..."
    if ! run cargo fmt --all -- --check; then
        log "Code formatting failed!"
        exit 1
    fi

    log "Running Clippy..."
    if ! run cargo clippy --all-targets --all-features -- -D warnings; then
        log "Linting failed!"
        exit 1
    fi

    log "Code is clean and formatted!"
fi
