# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Test
cargo test               # Run all tests
cargo test <test_name>   # Run specific test by name
cargo test --test <file> # Run tests in specific file (e.g., cargo test --test respawn_tests)
RUST_LOG=debug cargo test <test_name> -- --nocapture  # Run with debug output

# Lint and format
cargo fmt                # Format code
cargo clippy             # Run linter

# Run
cargo run -- <args>      # Run with arguments (e.g., cargo run -- list)
RUST_LOG=debug cargo run -- <args>  # Run with debug logging
```

## Architecture Overview

TATL is a CLI task/time tracking tool built with Rust and SQLite. The core philosophy is "doing work, not managing work" - simple semantics focused on execution.

### Module Structure

```
src/
├── cli/           # Command parsing and execution
│   ├── commands.rs         # Main command dispatch (add, list, modify, on, off, etc.)
│   ├── commands_sessions.rs # Session subcommands (sessions list/modify/delete/report)
│   ├── parser.rs           # Task argument parsing (extracts project:, +tag, due:, etc.)
│   └── output.rs           # Table/JSON output formatting
├── models/        # Data structures (Task, Project, Session, Stack, Annotation)
├── repo/          # Database access layer - one repo per model
├── db/            # Connection management and migrations
├── filter/        # Filter expression parser and evaluator
├── respawn/       # Respawn rule parser and next-date generator
└── utils/         # Date/duration parsing, fuzzy matching
```

### Key Concepts

- **Work Queue**: Tasks are managed via a queue (stack). `queue[0]` is always "what's next". Commands: `enqueue`, `dequeue`, `on` (moves to queue[0] and starts timing).

- **Time Tracking**: `on`/`off` commands with break capture (`offon 14:30` = interrupted at 14:30, resuming now) and historical sessions (`onoff 09:00..12:00`).

- **Respawning**: Unlike recurrence, respawning creates a new task instance only when the current one is completed. Patterns: `daily`, `weekly`, `monthdays:1,15`, `every:3d`, etc.

- **Kanban Status**: Derived from task state: `proposed` (not queued, no sessions), `queued`, `paused` (has sessions but not queued), `NEXT` (queue[0]), `LIVE` (queue[0] + active session), `done`.

- **Immutable Events**: All task changes are recorded in an event log for audit trail.

### Data Flow

1. CLI parses command via clap → `cli/commands.rs`
2. Commands use repos (`repo/*.rs`) to read/write data
3. Repos interact with SQLite via `db/connection.rs`
4. Complex queries use filter engine (`filter/parser.rs` + `filter/evaluator.rs`)

### Testing

Tests use `AcceptanceTestContext` from `tests/acceptance_framework.rs` which:
- Creates a temp directory with isolated database
- Sets `HOME` env var to temp directory
- Provides Given/When/Then builder pattern for test setup

To run a single test with output visible:
```bash
cargo test test_name -- --nocapture
```
