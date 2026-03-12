use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};

use crate::{
    config::AppConfig,
    core::{catalog::Catalog, navi, pack::parse_pack_file},
    integration::{
        detect::EnvironmentInfo,
        install::{
            default_install_path, default_man_install_path, explicit_target, render_auto_install,
            render_doctor, render_install_for_target, render_man_page, write_install_block,
            write_plain_file,
        },
    },
    paths,
    state::UserState,
    tui,
};

#[derive(Debug, Parser)]
#[command(
    name = "bindfinder",
    version,
    about = "Terminal-first command reference browser",
    long_about = "bindfinder is a terminal-first command reference browser for SSH, tmux, and shell-heavy workflows.\n\nRun it with no subcommand to open the TUI. The TUI starts in search mode so you can type immediately. Press Esc to enter normal mode for vim-style actions, and / to return to search mode.\n\nUse subcommands to inspect config paths, validate packs, print integration snippets, and search packs from the command line.",
    after_help = "Examples:\n  bindfinder\n  bindfinder search tmux split pane\n  bindfinder doctor\n  bindfinder install auto --write\n  bindfinder reload\n  bindfinder install man --write\n  bindfinder config init\n\nTUI flow:\n  Type immediately to filter\n  Esc enters normal mode\n  / returns to search mode\n  Enter selects the current result"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Manage runtime configuration
    Config(ConfigArgs),
    /// Print an integration snippet for a target
    Install(InstallArgs),
    /// Re-apply the detected integration and reload tmux when possible
    Reload,
    /// Detect the current environment and show the recommended integration
    Doctor,
    /// Search packs from the command line
    Search(SearchArgs),
    /// List discovered tools or important paths
    List(ListArgs),
    /// Validate a YAML pack file
    Validate(ValidateArgs),
    /// Browse and import navi-style cheat repositories
    Navi {
        #[command(subcommand)]
        command: NaviCommand,
    },
    #[command(hide = true)]
    TmuxCapture {
        #[arg(value_name = "PANE_ID")]
        target: String,
        #[arg(long)]
        kill_pane: bool,
    },
    #[command(hide = true)]
    TmuxLaunch,
}

#[derive(Debug, ClapArgs)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Print an integration snippet for a target",
    long_about = "Print an integration snippet for a target.\n\nUse `auto` to let bindfinder pick the best target for the current environment."
)]
struct InstallArgs {
    #[arg(value_enum, help = "Integration target to render")]
    target: InstallTarget,
    #[arg(long, help = "Write the generated snippet to the default config file for the target")]
    write: bool,
    #[arg(long, value_name = "PATH", help = "Write the generated snippet to an explicit file path")]
    path: Option<PathBuf>,
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Search packs from the command line",
    long_about = "Search loaded built-in and local packs from the command line and print tab-separated results.\n\nEach result includes tool, title, type, keys, and command."
)]
struct SearchArgs {
    #[arg(required = true, help = "Search terms to match against loaded entries")]
    query: Vec<String>,
}

#[derive(Debug, ClapArgs)]
#[command(about = "List discovered tools or important paths")]
struct ListArgs {
    #[arg(value_enum, help = "What to list")]
    kind: ListKind,
}

#[derive(Debug, ClapArgs)]
#[command(about = "Validate a YAML pack file")]
struct ValidateArgs {
    #[arg(value_name = "PACK_PATH", help = "Path to a YAML pack file")]
    pack: PathBuf,
}

#[derive(Debug, Subcommand)]
enum NaviCommand {
    /// List featured navi cheat repositories
    Featured,
    /// List locally imported navi repositories
    List,
    /// Import a navi cheat repository into the local repo cache
    Import {
        #[arg(value_name = "REPO", help = "Repository URL, SSH URL, or owner/repo shorthand")]
        repo: String,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    #[command(about = "Write the default config file to the standard config path")]
    Init {
        #[arg(long, help = "Overwrite an existing config file")]
        force: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum InstallTarget {
    Auto,
    Tmux,
    Bash,
    Zsh,
    Fish,
    Man,
}

#[derive(Debug, Clone, ValueEnum)]
enum ListKind {
    Tools,
    Config,
    Sources,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None => tui::run(),
        Some(Command::Config(args)) => match args.command {
            ConfigCommand::Init { force } => {
                let config = AppConfig::default();
                let path = AppConfig::default_path().context("no config path could be determined")?;
                if path.exists() && !force {
                    println!("{}", path.display());
                    return Ok(());
                }
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, config.to_yaml_string()?)?;
                println!("{}", path.display());
                Ok(())
            }
        },
        Some(Command::Install(args)) => {
            let config = AppConfig::load()?;
            let env = EnvironmentInfo::detect();
            let target_name = match args.target {
                InstallTarget::Auto => "auto",
                InstallTarget::Tmux => "tmux",
                InstallTarget::Bash => "bash",
                InstallTarget::Zsh => "zsh",
                InstallTarget::Fish => "fish",
                InstallTarget::Man => "man",
            };
            if matches!(args.target, InstallTarget::Man) {
                let man_page = render_man_page();
                if args.write {
                    let path = if let Some(path) = args.path {
                        path
                    } else {
                        default_man_install_path()
                            .context("no default man page path could be determined")?
                    };
                    write_plain_file(&path, man_page)?;
                    println!("{}", path.display());
                } else {
                    println!("{man_page}");
                }
                Ok(())
            } else {
                let target = explicit_target(target_name, &env, &config);
                let snippet = if matches!(args.target, InstallTarget::Auto) {
                    render_auto_install(&config, &env)
                } else {
                    render_install_for_target(&config, &target)
                };

                if args.write {
                    let path = if let Some(path) = args.path {
                        path
                    } else {
                        default_install_path(&target)
                            .context("no default install path for this target")?
                    };
                    write_install_block(&path, &snippet)?;
                    println!("{}", path.display());
                } else {
                    println!("{snippet}");
                }
                Ok(())
            }
        }
        Some(Command::Reload) => {
            let config = AppConfig::load()?;
            let env = EnvironmentInfo::detect();
            let target = env.choose_target(&config);

            match &target {
                crate::integration::detect::IntegrationTarget::Plain => {
                    println!("no reload action for the current environment");
                    println!("run: bindfinder install auto --write");
                }
                crate::integration::detect::IntegrationTarget::Terminal(_) => {
                    println!("terminal integration is snippet-only");
                    println!("run: bindfinder install auto");
                }
                _ => {
                    let snippet = render_install_for_target(&config, &target);
                    let path = default_install_path(&target)
                        .context("no default install path for this target")?;
                    write_install_block(&path, &snippet)?;

                    match &target {
                        crate::integration::detect::IntegrationTarget::Tmux => {
                            let status = ProcessCommand::new("tmux")
                                .arg("source-file")
                                .arg(&path)
                                .status();
                            match status {
                                Ok(status) if status.success() => {
                                    println!("reloaded tmux config: {}", path.display());
                                }
                                Ok(_) | Err(_) => {
                                    println!("updated tmux config: {}", path.display());
                                    println!("run: tmux source-file {}", path.display());
                                }
                            }
                        }
                        crate::integration::detect::IntegrationTarget::Shell(shell) => {
                            println!("updated shell config: {}", path.display());
                            println!("reload command: {}", shell_reload_hint(shell, &path));
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        }
        Some(Command::Doctor) => {
            let config = AppConfig::load()?;
            let env = EnvironmentInfo::detect();
            println!("{}", render_doctor(&config, &env));
            Ok(())
        }
        Some(Command::Search(args)) => {
            let catalog = Catalog::load_all()?;
            let state = UserState::load().unwrap_or_default();
            let query = args.query.join(" ");
            for item in catalog.filter_with_state(&query, &state, false, false) {
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
        Some(Command::List(args)) => {
            match args.kind {
                ListKind::Tools => {
                    let catalog = Catalog::load_all()?;
                    for tool in catalog.tools() {
                        println!("{tool}");
                    }
                }
                ListKind::Config => {
                    if let Some(path) = AppConfig::default_path() {
                        println!("{}", path.display());
                    }
                }
                ListKind::Sources => {
                    if let Some(dir) = Catalog::default_pack_dir() {
                        println!("{}", dir.display());
                    }
                }
            }
            Ok(())
        }
        Some(Command::Validate(args)) => {
            let parsed = parse_pack_file(&args.pack)?;
            println!(
                "valid\t{}\t{}\t{} entries",
                parsed.pack.id,
                parsed.pack.tool,
                parsed.entries.len()
            );
            Ok(())
        }
        Some(Command::Navi { command }) => match command {
            NaviCommand::Featured => {
                for repo in navi::featured_repos() {
                    println!("{repo}");
                }
                Ok(())
            }
            NaviCommand::List => {
                if let Some(dir) = Catalog::default_navi_repo_dir() {
                    if dir.exists() {
                        for entry in std::fs::read_dir(&dir)? {
                            let path = entry?.path();
                            if path.is_dir() {
                                println!("{}", path.display());
                            }
                        }
                    }
                }
                Ok(())
            }
            NaviCommand::Import { repo } => {
                let dir = Catalog::default_navi_repo_dir()
                    .context("no navi repo directory could be determined")?;
                std::fs::create_dir_all(&dir)?;
                let remote = normalize_repo_url(&repo);
                let target = dir.join(repo_dir_name(&repo));

                let status = if target.exists() {
                    ProcessCommand::new("git")
                        .arg("-C")
                        .arg(&target)
                        .arg("pull")
                        .arg("--ff-only")
                        .status()?
                } else {
                    ProcessCommand::new("git")
                        .arg("clone")
                        .arg("--depth")
                        .arg("1")
                        .arg(&remote)
                        .arg(&target)
                        .status()?
                };

                if !status.success() {
                    anyhow::bail!("git operation failed for {}", remote);
                }

                println!("{}", target.display());
                Ok(())
            }
        },
        Some(Command::TmuxCapture { target, kill_pane }) => {
            log_tmux_capture(&format!("start target={target} kill_pane={kill_pane}"))?;
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("system clock before unix epoch")?
                .as_nanos();
            let output_path = env::temp_dir().join(format!("bindfinder-{stamp}.out"));
            env::set_var("BINDFINDER_OUTPUT_FILE", &output_path);
            log_tmux_capture(&format!("output_path={}", output_path.display()))?;

            let tui_result = tui::run();
            log_tmux_capture(&format!("tui_result_ok={}", tui_result.is_ok()))?;
            let selected = fs::read_to_string(&output_path).unwrap_or_default();
            log_tmux_capture(&format!("selected_raw={selected:?}"))?;
            let _ = fs::remove_file(&output_path);
            tui_result?;

            let selected = selected.trim_end_matches(['\r', '\n']);
            log_tmux_capture(&format!("selected_trimmed={selected:?}"))?;
            if !selected.is_empty() {
                let set_status = ProcessCommand::new("tmux")
                    .arg("set-buffer")
                    .arg("--")
                    .arg(selected)
                    .status()?;
                log_tmux_capture(&format!("set_buffer_status={set_status}"))?;
                if !set_status.success() {
                    anyhow::bail!("tmux set-buffer failed");
                }

                let paste_status = ProcessCommand::new("tmux")
                    .arg("paste-buffer")
                    .arg("-p")
                    .arg("-d")
                    .arg("-t")
                    .arg(&target)
                    .status()?;
                log_tmux_capture(&format!("paste_buffer_status={paste_status}"))?;
                if !paste_status.success() {
                    anyhow::bail!("tmux paste-buffer failed for {}", target);
                }
            } else {
                log_tmux_capture("selected_trimmed_empty=true")?;
            }

            if kill_pane {
                let kill_status = ProcessCommand::new("tmux").arg("kill-pane").status()?;
                log_tmux_capture(&format!("kill_pane_status={kill_status}"))?;
                if !kill_status.success() {
                    anyhow::bail!("tmux kill-pane failed");
                }
            }

            log_tmux_capture("done")?;
            Ok(())
        }
        Some(Command::TmuxLaunch) => {
            let target = ProcessCommand::new("tmux")
                .arg("display-message")
                .arg("-p")
                .arg("#{pane_id}")
                .output()?;
            if !target.status.success() {
                anyhow::bail!("tmux display-message failed");
            }
            let target = String::from_utf8(target.stdout)
                .context("tmux pane id was not valid utf-8")?
                .trim()
                .to_string();
            if target.is_empty() {
                anyhow::bail!("tmux pane id was empty");
            }

            let exe = env::current_exe().context("could not resolve bindfinder binary path")?;
            let status = ProcessCommand::new("tmux")
                .arg("split-window")
                .arg("-v")
                .arg("-p")
                .arg("40")
                .arg(exe)
                .arg("tmux-capture")
                .arg(&target)
                .arg("--kill-pane")
                .status()?;
            if !status.success() {
                anyhow::bail!("tmux split-window failed");
            }
            Ok(())
        }
    }
}

fn log_tmux_capture(message: &str) -> Result<()> {
    if !tmux_debug_enabled() {
        return Ok(());
    }

    let path = debug_log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?
        .as_secs();

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "[{timestamp}] {message}")?;
    Ok(())
}

fn debug_log_path() -> PathBuf {
    if let Some(path) = env::var_os("BINDFINDER_DEBUG_LOG") {
        return PathBuf::from(path);
    }

    if let Some(path) = paths::bindfinder_cache_file("tmux-capture.log") {
        return path;
    }

    env::temp_dir().join("bindfinder-tmux-capture.log")
}

fn tmux_debug_enabled() -> bool {
    if env::var_os("BINDFINDER_DEBUG_LOG").is_some() {
        return true;
    }

    AppConfig::load()
        .map(|config| config.integration.tmux.debug)
        .unwrap_or(false)
}

fn normalize_repo_url(repo: &str) -> String {
    if repo.contains("://") || repo.starts_with("git@") {
        repo.to_string()
    } else {
        format!("https://github.com/{repo}")
    }
}

fn repo_dir_name(repo: &str) -> String {
    let trimmed = repo.trim_end_matches('/');
    let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    name.strip_suffix(".git").unwrap_or(name).to_string()
}

fn shell_reload_hint(shell: &crate::integration::detect::ShellKind, path: &std::path::Path) -> String {
    match shell {
        crate::integration::detect::ShellKind::Fish => format!("source {}", path.display()),
        crate::integration::detect::ShellKind::Bash | crate::integration::detect::ShellKind::Zsh => {
            format!("source {}", path.display())
        }
        crate::integration::detect::ShellKind::Unknown(_) => format!("source {}", path.display()),
    }
}
