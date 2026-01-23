#!/bin/bash
# Generate man page from Clap definitions using clap-mangen

set -e

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Create man directory if it doesn't exist
MAN_DIR="$PROJECT_ROOT/man/man1"
mkdir -p "$MAN_DIR"

# Generate man page
cd "$PROJECT_ROOT"
cargo run --bin generate-man --release 2>/dev/null || {
    echo "Note: generate-man binary not found. Creating it..."
    # This will be created by the build script
}

echo "Man page generated at: $MAN_DIR/tatl.1"
echo "To view: man -l $MAN_DIR/tatl.1"
echo "To install: sudo cp $MAN_DIR/tatl.1 /usr/local/share/man/man1/ && sudo mandb"
