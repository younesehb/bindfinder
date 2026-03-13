use std::process::Command as ProcessCommand;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};

use crate::{
    config::AppConfig,
    core::{catalog::Catalog, navi, pack::parse_pack_file},
    integration::{
        detect::EnvironmentInfo,
        install::{
            default_install_path, default_man_install_path, explicit_target, remove_install_block,
            render_auto_install, render_doctor, render_install_for_target, render_man_page,
            write_install_block, write_plain_file,
        },
    },
    paths,
    state::UserState,
    tui, update,
};

#[derive(Debug, Parser)]
#[command(
    name = "bindfinder",
    version,
    about = "Terminal-first command reference browser",
    long_about = "bindfinder is a terminal-first command reference browser for SSH, tmux, and shell-heavy workflows.\n\nRun it with no subcommand to open the TUI. The TUI starts in search mode so you can type immediately. Press Esc to enter normal mode for vim-style actions, and / to return to search mode.\n\nUse subcommands to inspect config paths, validate packs, print integration snippets, and search packs from the command line.",
    after_help = "Examples:\n  bindfinder\n  bindfinder search tmux split pane\n  bindfinder doctor\n  bindfinder update\n  bindfinder install auto --write\n  bindfinder reload\n  bindfinder install man --write\n  bindfinder config\n  bindfinder config init\n  bindfinder config validate\n\nTUI flow:\n  Type immediately to filter\n  Esc enters normal mode\n  / returns to search mode\n  Enter selects the current result"
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
    /// Remove bindfinder integrations and installed files
    Uninstall(UninstallArgs),
    /// Re-apply the detected integration and reload tmux when possible
    Reload,
    /// Detect the current environment and show the recommended integration
    Doctor(DoctorArgs),
    /// Check for a newer release or update to it
    Update(UpdateArgs),
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
    command: Option<ConfigCommand>,
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Print an integration snippet for a target",
    long_about = "Print an integration snippet for a target.\n\nUse `auto` to let bindfinder pick the best target for the current environment."
)]
struct InstallArgs {
    #[arg(value_enum, help = "Integration target to render")]
    target: InstallTarget,
    #[arg(
        long,
        help = "Write the generated snippet to the default config file for the target"
    )]
    write: bool,
    #[arg(
        long,
        value_name = "PATH",
        help = "Write the generated snippet to an explicit file path"
    )]
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
#[command(
    about = "Check for a newer release or update to it",
    long_about = "Check for a newer bindfinder release on GitHub. By default this command installs the latest release when one is available. Pass --check to only report the current status."
)]
struct UpdateArgs {
    #[arg(long, help = "Only check whether a newer release exists")]
    check: bool,
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Detect the current environment and show the recommended integration",
    long_about = "Detect the current environment and show the recommended integration.\n\nBy default this prints a concise summary. Pass --snippet to also print the generated integration snippet."
)]
struct DoctorArgs {
    #[arg(long, help = "Also print the generated integration snippet")]
    snippet: bool,
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Remove bindfinder integrations and installed files",
    long_about = "Remove bindfinder-managed shell/tmux blocks and installed files.\n\nBy default this removes the binary, man page, and managed integration blocks. Pass --purge-data to also remove user config, state, packs, repos, and cache files."
)]
struct UninstallArgs {
    #[arg(
        long,
        help = "Also remove config, state, packs, repos, and cache files"
    )]
    purge_data: bool,
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
        #[arg(
            value_name = "REPO",
            help = "Repository URL, SSH URL, or owner/repo shorthand"
        )]
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
    #[command(about = "Validate the current config file and print a clear result")]
    Validate,
}

#[derive(Debug, Clone, ValueEnum)]
enum InstallTarget {
    Auto,
    All,
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
            None => open_config_in_editor(),
            Some(ConfigCommand::Init { force }) => {
                let config = AppConfig::default();
                let path =
                    AppConfig::default_path().context("no config path could be determined")?;
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
            Some(ConfigCommand::Validate) => {
                let (_config, path) = load_config_for_command()?;
                if let Some(path) = path {
                    println!("valid config: {}", path.display());
                } else {
                    println!("valid config: defaults");
                }
                Ok(())
            }
        },
        Some(Command::Install(args)) => {
            let (config, _) = load_config_for_command()?;
            let env = EnvironmentInfo::detect();
            let target_name = match args.target {
                InstallTarget::Auto => "auto",
                InstallTarget::All => "all",
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
            } else if matches!(args.target, InstallTarget::All) {
                let targets = collect_applicable_targets(&env, &config);
                if targets.is_empty() {
                    println!("no supported integration targets detected");
                    return Ok(());
                }

                for target in targets {
                    let snippet = render_install_for_target(&config, &target);
                    let path = default_install_path(&target)
                        .context("no default install path for this target")?;
                    if args.write {
                        write_install_block(&path, &snippet)?;
                        println!("{}", path.display());
                    } else {
                        println!("# {}", path.display());
                        println!("{snippet}");
                    }
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
        Some(Command::Uninstall(args)) => {
            let config = AppConfig::load().unwrap_or_default();
            let env = EnvironmentInfo::detect();
            let mut removed_any = false;

            for target in collect_supported_uninstall_targets(&env, &config) {
                if let Some(path) = default_install_path(&target) {
                    if remove_install_block(&path)? {
                        println!("removed integration block: {}", path.display());
                        removed_any = true;
                    }
                }
            }

            if let Some(bin_root) = paths::home_dir().map(|home| home.join(".local").join("bin")) {
                let binary = bin_root.join("bindfinder");
                if binary.exists() {
                    fs::remove_file(&binary)
                        .with_context(|| format!("failed to remove {}", binary.display()))?;
                    println!("removed binary: {}", binary.display());
                    removed_any = true;
                }
            }

            if let Some(path) = default_man_install_path() {
                if path.exists() {
                    fs::remove_file(&path)
                        .with_context(|| format!("failed to remove {}", path.display()))?;
                    println!("removed man page: {}", path.display());
                    removed_any = true;
                }
            }

            if args.purge_data {
                for path in uninstall_data_paths() {
                    if path.exists() {
                        if path.is_dir() {
                            fs::remove_dir_all(&path).with_context(|| {
                                format!("failed to remove directory {}", path.display())
                            })?;
                        } else {
                            fs::remove_file(&path).with_context(|| {
                                format!("failed to remove file {}", path.display())
                            })?;
                        }
                        println!("removed data: {}", path.display());
                        removed_any = true;
                    }
                }
            }

            if !removed_any {
                println!("nothing to remove");
            }

            Ok(())
        }
        Some(Command::Reload) => perform_reload(),
        Some(Command::Doctor(args)) => {
            let (config, _) = load_config_for_command()?;
            let env = EnvironmentInfo::detect();
            println!("{}", render_doctor(&config, &env, args.snippet));
            Ok(())
        }
        Some(Command::Update(args)) => {
            let current = env!("CARGO_PKG_VERSION");
            if args.check {
                match update::check_now(current)? {
                    Some(info) => {
                        println!(
                            "update available: {} -> {}\nrun: bindfinder update\nrelease: {}",
                            info.current_version, info.latest_version, info.release_url
                        );
                    }
                    None => {
                        println!("bindfinder is up to date ({current})");
                    }
                }
                return Ok(());
            }

            match update::perform_update(current)? {
                Some(info) => {
                    println!(
                        "updated bindfinder: {} -> {}",
                        info.current_version, info.latest_version
                    );
                    println!("run: bindfinder reload");
                }
                None => {
                    println!("bindfinder is up to date ({current})");
                }
            }
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

fn open_config_in_editor() -> Result<()> {
    let config = AppConfig::default();
    let path = AppConfig::default_path().context("no config path could be determined")?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, config.to_yaml_string()?)?;
    }

    if let Some(swap_path) = vim_swap_path(&path) {
        if swap_path.exists() {
            fs::remove_file(&swap_path).with_context(|| {
                format!("failed to remove Vim swap file {}", swap_path.display())
            })?;
        }
    }

    let editor = preferred_editor()
        .context("no editor found; set VISUAL or EDITOR, or install one of: nvim, vim, nano, vi")?;

    let status = ProcessCommand::new(&editor.program)
        .args(&editor.args)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to start editor '{}'", editor.program))?;

    if !status.success() {
        anyhow::bail!(
            "editor '{}' exited with status {}",
            editor.program,
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }

    let (_config, path) = load_config_for_command()?;
    if let Some(path) = path {
        println!("validated config: {}", path.display());
    } else {
        println!("validated config: defaults");
    }
    perform_reload()?;

    Ok(())
}

fn perform_reload() -> Result<()> {
    let (config, _) = load_config_for_command()?;
    let env = EnvironmentInfo::detect();
    let targets = collect_applicable_targets(&env, &config);

    if targets.is_empty() {
        println!("no reload action for the current environment");
        println!("run: bindfinder install auto --write");
        return Ok(());
    }

    for target in targets {
        let snippet = render_install_for_target(&config, &target);
        let path =
            default_install_path(&target).context("no default install path for this target")?;
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

    Ok(())
}

struct EditorCommand {
    program: String,
    args: Vec<String>,
}

fn preferred_editor() -> Option<EditorCommand> {
    if let Some(editor) = env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return Some(parse_editor_command(&editor));
    }
    if let Some(editor) = env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return Some(parse_editor_command(&editor));
    }

    for candidate in ["nvim", "vim", "nano", "vi"] {
        if command_exists(candidate) {
            return Some(EditorCommand {
                program: candidate.to_string(),
                args: Vec::new(),
            });
        }
    }

    None
}

fn parse_editor_command(command: &str) -> EditorCommand {
    let mut parts = command.split_whitespace();
    let program = parts.next().unwrap_or(command).to_string();
    let args = parts.map(ToString::to_string).collect();
    EditorCommand { program, args }
}

fn command_exists(program: &str) -> bool {
    ProcessCommand::new("sh")
        .arg("-lc")
        .arg(format!("command -v {} >/dev/null 2>&1", program))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn vim_swap_path(path: &PathBuf) -> Option<PathBuf> {
    let parent = path.parent()?;
    let file_name = path.file_name()?.to_str()?;
    Some(parent.join(format!(".{file_name}.swp")))
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

fn shell_reload_hint(
    shell: &crate::integration::detect::ShellKind,
    path: &std::path::Path,
) -> String {
    match shell {
        crate::integration::detect::ShellKind::Fish => format!("source {}", path.display()),
        crate::integration::detect::ShellKind::Bash
        | crate::integration::detect::ShellKind::Zsh => {
            format!("source {}", path.display())
        }
        crate::integration::detect::ShellKind::Unknown(_) => format!("source {}", path.display()),
    }
}

fn load_config_for_command() -> Result<(AppConfig, Option<PathBuf>)> {
    AppConfig::load_with_path().map_err(|err| {
        if let Some(path) = AppConfig::default_path() {
            err.context(format!("invalid config: {}", path.display()))
        } else {
            err.context("invalid config")
        }
    })
}

fn collect_supported_uninstall_targets(
    env: &EnvironmentInfo,
    config: &AppConfig,
) -> Vec<crate::integration::detect::IntegrationTarget> {
    let mut targets = collect_applicable_targets(env, config);
    for fallback in [
        crate::integration::detect::IntegrationTarget::Tmux,
        crate::integration::detect::IntegrationTarget::Shell(
            crate::integration::detect::ShellKind::Bash,
        ),
        crate::integration::detect::IntegrationTarget::Shell(
            crate::integration::detect::ShellKind::Zsh,
        ),
        crate::integration::detect::IntegrationTarget::Shell(
            crate::integration::detect::ShellKind::Fish,
        ),
    ] {
        if !targets
            .iter()
            .any(|target| same_target_kind(target, &fallback))
        {
            targets.push(fallback);
        }
    }
    targets
}

fn same_target_kind(
    left: &crate::integration::detect::IntegrationTarget,
    right: &crate::integration::detect::IntegrationTarget,
) -> bool {
    match (left, right) {
        (
            crate::integration::detect::IntegrationTarget::Tmux,
            crate::integration::detect::IntegrationTarget::Tmux,
        ) => true,
        (
            crate::integration::detect::IntegrationTarget::Shell(left),
            crate::integration::detect::IntegrationTarget::Shell(right),
        ) => std::mem::discriminant(left) == std::mem::discriminant(right),
        _ => false,
    }
}

fn uninstall_data_paths() -> Vec<PathBuf> {
    let mut paths_to_remove = Vec::new();

    if let Some(path) = AppConfig::default_path() {
        paths_to_remove.push(path);
    }
    if let Some(path) = crate::state::default_path() {
        paths_to_remove.push(path);
    }
    if let Some(path) = Catalog::default_pack_dir() {
        paths_to_remove.push(path);
    }
    if let Some(path) = Catalog::default_navi_repo_dir() {
        paths_to_remove.push(path);
    }
    if let Some(path) = paths::bindfinder_cache_file("tmux-capture.log") {
        paths_to_remove.push(path);
    }

    paths_to_remove
}

fn collect_applicable_targets(
    env: &EnvironmentInfo,
    config: &AppConfig,
) -> Vec<crate::integration::detect::IntegrationTarget> {
    let mut targets = Vec::new();

    if config.integration.tmux.enabled && env.inside_tmux {
        targets.push(crate::integration::detect::IntegrationTarget::Tmux);
    }

    if config.integration.shell.enabled {
        if let Some(shell) = env.shell.clone() {
            match shell {
                crate::integration::detect::ShellKind::Bash
                | crate::integration::detect::ShellKind::Zsh
                | crate::integration::detect::ShellKind::Fish => {
                    targets.push(crate::integration::detect::IntegrationTarget::Shell(shell));
                }
                crate::integration::detect::ShellKind::Unknown(_) => {}
            }
        }
    }

    if targets.is_empty() {
        match env.choose_target(config) {
            crate::integration::detect::IntegrationTarget::Tmux => {
                targets.push(crate::integration::detect::IntegrationTarget::Tmux)
            }
            crate::integration::detect::IntegrationTarget::Shell(shell) => match shell {
                crate::integration::detect::ShellKind::Bash
                | crate::integration::detect::ShellKind::Zsh
                | crate::integration::detect::ShellKind::Fish => {
                    targets.push(crate::integration::detect::IntegrationTarget::Shell(shell))
                }
                crate::integration::detect::ShellKind::Unknown(_) => {}
            },
            _ => {}
        }
    }

    targets
}
