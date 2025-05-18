//! Maestro binary entry point — parses arguments and dispatches.
//! All logic lives in the `maestro` library crate so it can be tested.

use anyhow::Result;
use clap::Parser;
use maestro::cli::{run, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}
