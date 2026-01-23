#!/bin/bash
# Install man page for current user (no sudo required)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

# Generate man page if it doesn't exist
if [ ! -f "$PROJECT_ROOT/man/man1/tatl.1" ]; then
    echo "Generating man page..."
    cd "$PROJECT_ROOT"
    cargo run --bin generate-man > /dev/null 2>&1 || {
        echo "Error: Failed to generate man page. Make sure you've built the project with 'cargo build'"
        exit 1
    }
fi

# Install for current user
mkdir -p ~/.local/share/man/man1
cp "$PROJECT_ROOT/man/man1/tatl.1" ~/.local/share/man/man1/

echo "Man page installed to ~/.local/share/man/man1/tatl.1"

# Check if MANPATH needs to be updated
SHELL_CONFIG=""
if [ -f ~/.bashrc ]; then
    SHELL_CONFIG=~/.bashrc
elif [ -f ~/.zshrc ]; then
    SHELL_CONFIG=~/.zshrc
fi

if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q 'MANPATH.*\.local/share/man' "$SHELL_CONFIG" 2>/dev/null; then
        echo "" >> "$SHELL_CONFIG"
        echo "# Add local man pages to MANPATH" >> "$SHELL_CONFIG"
        echo 'export MANPATH="$HOME/.local/share/man:$MANPATH"' >> "$SHELL_CONFIG"
        echo "Added MANPATH to $SHELL_CONFIG"
        echo "Run: source $SHELL_CONFIG"
    else
        echo "MANPATH already configured in $SHELL_CONFIG"
    fi
fi

echo ""
echo "To use the man page now, run:"
echo "  export MANPATH=\"\$HOME/.local/share/man:\$MANPATH\""
echo "  man tatl"
echo ""
echo "Or reload your shell config:"
echo "  source $SHELL_CONFIG"
