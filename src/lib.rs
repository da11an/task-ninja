//! Task Ninja - A powerful command-line task management tool
//!
//! This library provides the core functionality for Task Ninja, including:
//! - Database operations and migrations
//! - Data models for tasks, projects, sessions, and more
//! - Repository layer for data access
//! - CLI command parsing and execution
//! - Filter expression parsing and evaluation
//! - Recurrence rule parsing and generation
//! - Date/time and duration utilities
//!
//! # Example
//!
//! ```no_run
//! use task_ninja::cli::run;
//!
//! fn main() {
//!     if let Err(e) = run() {
//!         eprintln!("Error: {}", e);
//!         std::process::exit(1);
//!     }
//! }
//! ```

pub mod db;
pub mod models;
pub mod repo;
pub mod cli;
pub mod utils;
pub mod filter;
pub mod recur;