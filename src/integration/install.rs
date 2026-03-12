use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    config::AppConfig,
    integration::detect::{EnvironmentInfo, IntegrationTarget, ShellKind, TerminalKind},
    paths,
};

const START_MARKER: &str = "# >>> bindfinder >>>";
const END_MARKER: &str = "# <<< bindfinder <<<";

pub fn render_auto_install(config: &AppConfig, env: &EnvironmentInfo) -> String {
    let target = env.choose_target(config);
    render_install_for_target(config, &target)
}

pub fn render_install_for_target(config: &AppConfig, target: &IntegrationTarget) -> String {
    match target {
        IntegrationTarget::Tmux => render_tmux(config),
        IntegrationTarget::Shell(shell) => render_shell(config, shell),
        IntegrationTarget::Terminal(terminal) => render_terminal(config, terminal),
        IntegrationTarget::Plain => render_plain(),
    }
}

pub fn render_man_page() -> &'static str {
    include_str!("../../man/bindfinder.1")
}

pub fn render_doctor(config: &AppConfig, env: &EnvironmentInfo) -> String {
    let target = env.choose_target(config);
    let shell = env
        .shell
        .as_ref()
        .map(format_shell)
        .unwrap_or_else(|| "unknown".to_string());
    let terminal = env
        .terminal
        .as_ref()
        .map(format_terminal)
        .unwrap_or_else(|| "unknown".to_string());

    let lines = vec![
        format!("mode: {}", format_mode(&config.integration.mode)),
        format!("inside_tmux: {}", env.inside_tmux),
        format!("over_ssh: {}", env.over_ssh),
        format!("shell: {}", shell),
        format!("terminal: {}", terminal),
        format!("launch_key: {}", config.integration.launch_key),
        format!("selected_target: {}", format_target(&target)),
        String::new(),
        "install_snippet:".to_string(),
        render_install_for_target(config, &target),
    ];

    lines.join("\n")
}

fn render_tmux(config: &AppConfig) -> String {
    let bindfinder = tmux_bindfinder_path();

    if config.integration.tmux.use_popup {
        format!(
            "bind-key {} run-shell \"{} tmux-launch\"",
            config.integration.tmux.key,
            bindfinder
        )
    } else {
        format!(
            "bind-key {} run-shell \"{} tmux-launch\"",
            config.integration.tmux.key,
            bindfinder
        )
    }
}

fn render_shell(config: &AppConfig, shell: &ShellKind) -> String {
    match shell {
        ShellKind::Bash => render_bash_shell(config),
        ShellKind::Zsh => render_zsh_shell(config),
        ShellKind::Fish => render_fish_shell(config),
        ShellKind::Unknown(name) => format!(
            "# shell '{}' is not directly supported yet\nbindfinder",
            name
        ),
    }
}

fn render_terminal(config: &AppConfig, terminal: &TerminalKind) -> String {
    let Some(key) = terminal_binding_key(&config.integration.launch_key) else {
        return format!(
            "# terminal launch key '{}' is not representable as a single terminal shortcut\n# requested launch key: {}",
            config.integration.launch_key,
            config.integration.launch_key
        );
    };

    match terminal {
        TerminalKind::WezTerm => format!(
            "keys = {{ {{ key = '{}', mods = 'CTRL', action = wezterm.action.SpawnCommandInNewTab {{ args = {{ 'bindfinder' }} }} }} }} -- requested launch key: {}",
            key,
            config.integration.launch_key
        ),
        TerminalKind::Kitty => format!(
            "map ctrl+{} launch --type=overlay bindfinder\n# requested launch key: {}",
            key,
            config.integration.launch_key
        ),
        TerminalKind::Iterm2 => format!(
            "# configure iTerm2 to send command: bindfinder\n# requested launch key: {}",
            config.integration.launch_key
        ),
        TerminalKind::Unknown(name) => format!(
            "# terminal '{}' is not directly supported yet\nbindfinder",
            name
        ),
    }
}

fn render_plain() -> String {
    "bindfinder".to_string()
}

fn tmux_bindfinder_path() -> String {
    env::current_exe()
        .ok()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "bindfinder".to_string())
}

pub fn explicit_target(target: &str, env: &EnvironmentInfo, config: &AppConfig) -> IntegrationTarget {
    match target {
        "auto" => env.choose_target(config),
        "tmux" => IntegrationTarget::Tmux,
        "bash" => IntegrationTarget::Shell(ShellKind::Bash),
        "zsh" => IntegrationTarget::Shell(ShellKind::Zsh),
        "fish" => IntegrationTarget::Shell(ShellKind::Fish),
        other => IntegrationTarget::Shell(ShellKind::Unknown(other.to_string())),
    }
}

pub fn default_install_path(target: &IntegrationTarget) -> Option<PathBuf> {
    let home = paths::home_dir()?;

    match target {
        IntegrationTarget::Tmux => Some(home.join(".tmux.conf")),
        IntegrationTarget::Shell(ShellKind::Bash) => Some(home.join(".bashrc")),
        IntegrationTarget::Shell(ShellKind::Zsh) => Some(home.join(".zshrc")),
        IntegrationTarget::Shell(ShellKind::Fish) => Some(home.join(".config").join("fish").join("config.fish")),
        _ => None,
    }
}

pub fn default_man_install_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("BINDFINDER_MANPAGE_DIR") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path.join("bindfinder.1"));
        }
    }

    paths::local_share_root().map(|root| root.join("man").join("man1").join("bindfinder.1"))
}

pub fn write_plain_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn write_install_block(path: &Path, snippet: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    let managed = managed_block(snippet);
    let new_content = replace_or_append_managed_block(&existing, &managed);

    fs::write(path, new_content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn managed_block(snippet: &str) -> String {
    format!("{START_MARKER}\n{snippet}\n{END_MARKER}\n")
}

fn replace_or_append_managed_block(existing: &str, managed: &str) -> String {
    if let Some(start) = existing.find(START_MARKER) {
        if let Some(end_rel) = existing[start..].find(END_MARKER) {
            let end = start + end_rel + END_MARKER.len();
            let mut output = String::new();
            output.push_str(&existing[..start]);
            if !output.is_empty() && !output.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(managed);
            if end < existing.len() {
                let remainder = existing[end..].trim_start_matches('\n');
                if !remainder.is_empty() {
                    output.push('\n');
                    output.push_str(remainder);
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
            return output;
        }
    }

    let mut output = existing.trim_end().to_string();
    if !output.is_empty() {
        output.push_str("\n\n");
    }
    output.push_str(managed);
    output
}

fn format_shell(shell: &ShellKind) -> String {
    match shell {
        ShellKind::Bash => "bash".to_string(),
        ShellKind::Zsh => "zsh".to_string(),
        ShellKind::Fish => "fish".to_string(),
        ShellKind::Unknown(value) => value.clone(),
    }
}

fn format_terminal(terminal: &TerminalKind) -> String {
    match terminal {
        TerminalKind::WezTerm => "wezterm".to_string(),
        TerminalKind::Kitty => "kitty".to_string(),
        TerminalKind::Iterm2 => "iterm2".to_string(),
        TerminalKind::Unknown(value) => value.clone(),
    }
}

fn format_target(target: &IntegrationTarget) -> String {
    match target {
        IntegrationTarget::Tmux => "tmux".to_string(),
        IntegrationTarget::Shell(shell) => format!("shell:{}", format_shell(shell)),
        IntegrationTarget::Terminal(terminal) => format!("terminal:{}", format_terminal(terminal)),
        IntegrationTarget::Plain => "plain".to_string(),
    }
}

fn format_mode(mode: &crate::config::IntegrationMode) -> &'static str {
    match mode {
        crate::config::IntegrationMode::Auto => "auto",
        crate::config::IntegrationMode::Tmux => "tmux",
        crate::config::IntegrationMode::Shell => "shell",
        crate::config::IntegrationMode::Terminal => "terminal",
    }
}

fn render_bash_shell(config: &AppConfig) -> String {
    format!(
        "bindfinder_capture() {{\n  local cmd\n  cmd=\"$(bindfinder)\" || return\n  [ -n \"$cmd\" ] || return\n  printf '%s' \"$cmd\"\n}}\nbindfinder_widget() {{\n  local cmd\n  cmd=\"$(bindfinder_capture)\" || return\n  READLINE_LINE=\"${{READLINE_LINE:0:READLINE_POINT}}$cmd${{READLINE_LINE:READLINE_POINT}}\"\n  READLINE_POINT=$((READLINE_POINT + ${{#cmd}}))\n}}\nif [[ ${{BLE_VERSION-}} ]] && type ble-bind >/dev/null 2>&1; then\n  function ble/widget/bindfinder {{\n    local cmd\n    cmd=\"$(bindfinder_capture)\" || return\n    ble/widget/insert-string \"$cmd\"\n    ble/textarea#invalidate\n  }}\n  ble-bind -f '{}' bindfinder\nelse\n  bind -x '\"{}\":bindfinder_widget'\nfi\n# requested launch key: {}",
        ble_bash_binding(&config.integration.shell.binding),
        bash_binding(&config.integration.shell.binding),
        config.integration.shell.binding
    )
}

fn render_zsh_shell(config: &AppConfig) -> String {
    format!(
        "bindfinder-widget() {{\n  local cmd\n  cmd=\"$(bindfinder)\" || return\n  [[ -n \"$cmd\" ]] || return\n  LBUFFER+=\"$cmd\"\n}}\nzle -N bindfinder-widget\nbindkey '{}' bindfinder-widget\n# requested launch key: {}",
        zsh_binding(&config.integration.shell.binding),
        config.integration.shell.binding
    )
}

fn render_fish_shell(config: &AppConfig) -> String {
    format!(
        "function bindfinder_widget\n    set -l cmd (bindfinder)\n    or return\n    test -n \"$cmd\"; or return\n    commandline -i -- \"$cmd\"\nend\nbind {} bindfinder_widget",
        fish_binding(&config.integration.shell.binding)
    )
}

fn bash_binding(binding: &str) -> String {
    bash_like_binding(binding, false)
}

fn zsh_binding(binding: &str) -> String {
    binding
        .split_whitespace()
        .map(zsh_binding_token)
        .collect::<Vec<_>>()
        .join("")
}

fn fish_binding(binding: &str) -> String {
    binding
        .split_whitespace()
        .map(fish_binding_token)
        .collect::<Vec<_>>()
        .join("")
}

fn terminal_binding_key(binding: &str) -> Option<String> {
    if binding.split_whitespace().count() != 1 {
        return None;
    }

    match binding.trim().to_ascii_lowercase().as_str() {
        "ctrl-/" => Some("/".to_string()),
        "ctrl-*" => Some("*".to_string()),
        other if other.starts_with("ctrl-") && other.len() == 6 => Some(other[5..].to_string()),
        other if other.len() == 1 => Some(other.to_string()),
        _ => None,
    }
}

fn bash_like_binding(binding: &str, uppercase_ctrl: bool) -> String {
    let tokens: Vec<String> = binding
        .split_whitespace()
        .map(|token| bash_binding_token(token, uppercase_ctrl))
        .collect();
    if tokens.is_empty() {
        "\\C-g\\C-b".to_string()
    } else {
        tokens.join("")
    }
}

fn bash_binding_token(token: &str, uppercase_ctrl: bool) -> String {
    let token = token.trim().to_ascii_lowercase();
    match token.as_str() {
        "ctrl-/" => "\\C-_".to_string(),
        "ctrl-*" => "\\C-*".to_string(),
        other if other.starts_with("ctrl-") && other.len() == 6 => {
            let key = &other[5..];
            if uppercase_ctrl {
                format!("C-{}", key)
            } else {
                format!("\\C-{}", key)
            }
        }
        other if other.starts_with("alt-") && other.len() == 5 => {
            let key = &other[4..];
            if uppercase_ctrl {
                format!("M-{}", key)
            } else {
                format!("\\e{}", key)
            }
        }
        other if other.len() == 1 => other.to_string(),
        _ => {
            if uppercase_ctrl {
                "C-g C-b".to_string()
            } else {
                "\\C-g\\C-b".to_string()
            }
        }
    }
}

fn ble_bash_binding(binding: &str) -> String {
    let tokens: Vec<String> = binding
        .split_whitespace()
        .map(|token| bash_binding_token(token, true))
        .collect();
    if tokens.is_empty() {
        "C-g C-b".to_string()
    } else {
        tokens.join(" ")
    }
}

fn zsh_binding_token(token: &str) -> String {
    let token = token.trim().to_ascii_lowercase();
    match token.as_str() {
        "ctrl-/" => "^_".to_string(),
        "ctrl-*" => "^*".to_string(),
        other if other.starts_with("ctrl-") && other.len() == 6 => {
            format!("^{}", other[5..].to_ascii_uppercase())
        }
        other if other.starts_with("alt-") && other.len() == 5 => {
            format!("^[{}", &other[4..])
        }
        other if other.len() == 1 => other.to_string(),
        _ => "^G^B".to_string(),
    }
}

fn fish_binding_token(token: &str) -> String {
    let token = token.trim().to_ascii_lowercase();
    match token.as_str() {
        "ctrl-/" => "\\c/".to_string(),
        "ctrl-*" => "\\c*".to_string(),
        other if other.starts_with("ctrl-") && other.len() == 6 => {
            format!("\\c{}", &other[5..])
        }
        other if other.starts_with("alt-") && other.len() == 5 => {
            format!("\\e{}", &other[4..])
        }
        other if other.len() == 1 => other.to_string(),
        _ => "\\cg\\cb".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_managed_block() {
        let existing = "line1\n# >>> bindfinder >>>\nold\n# <<< bindfinder <<<\nline2\n";
        let updated = replace_or_append_managed_block(existing, &managed_block("new"));
        assert!(updated.contains("new"));
        assert!(!updated.contains("old\n"));
        assert!(updated.contains("line1"));
        assert!(updated.contains("line2"));
    }

    #[test]
    fn appends_managed_block_when_missing() {
        let updated = replace_or_append_managed_block("line1\n", &managed_block("new"));
        assert!(updated.contains("line1"));
        assert!(updated.contains("new"));
        assert!(updated.contains(START_MARKER));
    }

    #[test]
    fn renders_ctrl_star_bindings() {
        assert_eq!(bash_binding("ctrl-*"), "\\C-*");
        assert_eq!(zsh_binding("ctrl-*"), "^*");
        assert_eq!(fish_binding("ctrl-*"), "\\c*");
        assert_eq!(terminal_binding_key("ctrl-*"), Some("*".to_string()));
    }

    #[test]
    fn renders_multi_stroke_shell_bindings() {
        assert_eq!(bash_binding("ctrl-g ctrl-b"), "\\C-g\\C-b");
        assert_eq!(ble_bash_binding("ctrl-g ctrl-b"), "C-g C-b");
        assert_eq!(zsh_binding("ctrl-g ctrl-b"), "^G^B");
        assert_eq!(fish_binding("ctrl-g ctrl-b"), "\\cg\\cb");
        assert_eq!(terminal_binding_key("ctrl-g ctrl-b"), None);
    }
}
