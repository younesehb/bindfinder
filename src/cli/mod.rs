use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{
    config::AppConfig,
    core::{catalog::Catalog, pack::parse_pack_file},
    integration::{
        detect::EnvironmentInfo,
        install::{render_auto_install, render_doctor},
    },
    tui,
};

#[derive(Debug, Parser)]
#[command(name = "bindfinder", version, about = "Terminal-first command reference browser")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Install {
        target: String,
    },
    Doctor,
    Search { query: Vec<String> },
    List { kind: String },
    Validate { pack: String },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Init,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None => tui::run(),
        Some(Command::Config { command }) => match command {
            ConfigCommand::Init => {
                let config = AppConfig::default();
                let path = AppConfig::default_path().context("no config path could be determined")?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, config.to_yaml_string()?)?;
                println!("{}", path.display());
                Ok(())
            }
        },
        Some(Command::Install { target }) => {
            let config = AppConfig::load()?;
            let env = EnvironmentInfo::detect();
            if target == "auto" {
                println!("{}", render_auto_install(&config, &env));
            } else {
                println!("unsupported install target: {target}");
            }
            Ok(())
        }
        Some(Command::Doctor) => {
            let config = AppConfig::load()?;
            let env = EnvironmentInfo::detect();
            println!("{}", render_doctor(&config, &env));
            Ok(())
        }
        Some(Command::Search { query }) => {
            let catalog = Catalog::load_all()?;
            let query = query.join(" ");
            for item in catalog.filter(&query) {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    item.tool,
                    item.entry.title,
                    item.entry.entry_type,
                    item.entry.keys.as_deref().unwrap_or("-"),
                    item.entry.command.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
        Some(Command::List { kind }) => {
            if kind == "tools" {
                let catalog = Catalog::load_all()?;
                for tool in catalog.tools() {
                    println!("{tool}");
                }
            } else if kind == "config" {
                if let Some(path) = AppConfig::default_path() {
                    println!("{}", path.display());
                }
            } else if kind == "sources" {
                if let Some(dir) = Catalog::default_pack_dir() {
                    println!("{}", dir.display());
                }
            } else {
                println!("unsupported list kind: {kind}");
            }
            Ok(())
        }
        Some(Command::Validate { pack }) => {
            let path = PathBuf::from(pack);
            let parsed = parse_pack_file(&path)?;
            println!(
                "valid\t{}\t{}\t{} entries",
                parsed.pack.id,
                parsed.pack.tool,
                parsed.entries.len()
            );
            Ok(())
        }
    }
}
