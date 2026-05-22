#!/bin/bash

SILENT=false
PACKAGES=""

# Check for silent flag
if [ "$1" = "--silent" ]; then
    SILENT=true
    shift
fi

# Check if any arguments are provided
if [ $# -gt 0 ]; then
    echo "Linting specific packages: $@"
    
    # Convert all arguments into --package flags
    for pkg in "$@"; do
        # Verify each package exists in the workspace
        if ! cargo metadata --format-version 1 | grep -q "\"name\":\"$pkg\""; then
            if [ "$SILENT" = false ]; then
                echo "Error: Package '$pkg' not found in workspace"
            fi
            exit 1
        fi
        PACKAGES="$PACKAGES --package $pkg"
    done
    
    # Format code for specific packages
    if [ "$SILENT" = false ]; then echo "Running rustfmt..."; fi
    if [ "$SILENT" = true ]; then
        cargo fmt $PACKAGES -- --check 2>/dev/null
    else
        cargo fmt $PACKAGES -- --check
    fi
    
    # Lint code for specific packages only
    if [ "$SILENT" = false ]; then echo "Running Clippy..."; fi
    if [ "$SILENT" = true ]; then
        cargo clippy $PACKAGES --no-deps -- -D warnings 2>/dev/null
    else
        cargo clippy $PACKAGES --no-deps -- -D warnings
    fi
    
    # Check for errors
    if [ $? -ne 0 ]; then
        if [ "$SILENT" = false ]; then
            echo "Code formatting or linting failed!"
        fi
        exit 1
    else
        if [ "$SILENT" = false ]; then
            echo "Code is clean and formatted!"
        fi
    fi
else
    echo "Linting all crates in workspace"
    # Format code
    if [ "$SILENT" = false ]; then echo "Running rustfmt..."; fi
    if [ "$SILENT" = true ]; then
        cargo fmt --all -- --check 2>/dev/null
    else
        cargo fmt --all -- --check
    fi
    
    # Lint code
    if [ "$SILENT" = false ]; then echo "Running Clippy..."; fi
    if [ "$SILENT" = true ]; then
        cargo clippy --all-targets --all-features -- -D warnings 2>/dev/null
    else
        cargo clippy --all-targets --all-features -- -D warnings
    fi
    
    # Check for errors
    if [ $? -ne 0 ]; then
        if [ "$SILENT" = false ]; then
            echo "Code formatting or linting failed!"
        fi
        echo "Code formatting error!"
    else
        if [ "$SILENT" = false ]; then
            echo "Code is clean and formatted!"
        fi
    fi
fi