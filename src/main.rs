mod cli;
mod config;
mod core;
mod integration;
mod paths;
mod state;
mod tui;
mod update;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
