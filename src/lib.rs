//! Maestro — a terminal piano-learning companion.
//!
//! The crate is split into a library (this module tree) and a thin binary
//! (`main.rs`) so the logic can be unit- and integration-tested without a
//! terminal or MIDI hardware. Live MIDI output lives behind the `midi`
//! cargo feature; everything else builds with no system dependencies.

pub mod cli;
pub mod config;
pub mod data;
pub mod importer;
pub mod keyboard;
pub mod midi;
pub mod model;
pub mod music;
pub mod notes;
pub mod playlist;
pub mod practice;
pub mod progress;
pub mod songs;
pub mod theory;
pub mod tui;
pub mod user;

/// Crate version, surfaced by the `--version` flag and the TUI banner.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
