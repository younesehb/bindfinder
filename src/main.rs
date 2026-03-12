mod config;
mod cli;
mod core;
mod integration;
mod tui;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
