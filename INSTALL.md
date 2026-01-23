# Installation Guide

## Quick Install

```bash
# Build and install
cargo build --release
cargo install --path .
```

This installs the `tatl` binary to `~/.cargo/bin/tatl`.

## Note: No Conflicts with Taskwarrior

The `tatl` command does not conflict with Taskwarrior's `task` command, so you can use both tools simultaneously without any special configuration.

## Verify Installation

```bash
# Check if installed
which tatl
~/.cargo/bin/tatl status

# Check version
~/.cargo/bin/tatl --help
```

## Man Page Installation

After installing `tatl`, you can also install the man page for `man tatl` support.

### Quick Install (Recommended - User Installation)

```bash
# Run the installation script (automatically sets up MANPATH)
./install-man-user.sh

# Then reload your shell
source ~/.bashrc  # or source ~/.zshrc

# Now you can use:
man tatl
```

### Manual Installation

**For current user (no sudo required):**
```bash
# Generate the man page
./scripts/generate-man.sh

# Install to user directory
mkdir -p ~/.local/share/man/man1
cp man/man1/tatl.1 ~/.local/share/man/man1/

# Add to MANPATH (add to ~/.bashrc or ~/.zshrc):
echo 'export MANPATH="$HOME/.local/share/man:$MANPATH"' >> ~/.bashrc
source ~/.bashrc  # or source ~/.zshrc
```

**For system-wide installation (requires sudo):**
```bash
# Generate the man page
./scripts/generate-man.sh

# Install system-wide
sudo cp man/man1/tatl.1 /usr/local/share/man/man1/
sudo mandb
```

After installation, you can view the man page with:
```bash
man tatl
```

## Uninstall

```bash
cargo uninstall tatl
# Optionally remove man page:
# sudo rm /usr/local/share/man/man1/tatl.1
# sudo mandb
```

## Local Development (No Installation)

For testing without installing:

```bash
# Build release version
cargo build --release

# Use directly
./target/release/tatl status

# Or create a symlink in a local bin directory
mkdir -p ~/bin
ln -s $(pwd)/target/release/tatl ~/bin/tatl
export PATH="$HOME/bin:$PATH"
```

## Troubleshooting

**Problem:** `tatl` command not found after installation

**Solution:** Make sure `~/.cargo/bin` is in your PATH:
```bash
echo $PATH | grep cargo
```

If not, add it to your shell config file (`~/.bashrc` or `~/.zshrc`):
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc
```
