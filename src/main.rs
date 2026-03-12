mod config;
mod cli;
mod core;
mod integration;
mod paths;
mod state;
mod tui;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
