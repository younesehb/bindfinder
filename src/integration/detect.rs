use std::{env, path::Path};

use crate::config::{AppConfig, IntegrationMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrationTarget {
    Tmux,
    Shell(ShellKind),
    Terminal(TerminalKind),
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalKind {
    WezTerm,
    Kitty,
    Iterm2,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub inside_tmux: bool,
    pub over_ssh: bool,
    pub shell: Option<ShellKind>,
    pub terminal: Option<TerminalKind>,
}

impl EnvironmentInfo {
    pub fn detect() -> Self {
        Self {
            inside_tmux: env::var_os("TMUX").is_some(),
            over_ssh: env::var_os("SSH_CONNECTION").is_some() || env::var_os("SSH_TTY").is_some(),
            shell: detect_shell(),
            terminal: detect_terminal(),
        }
    }

    pub fn choose_target(&self, config: &AppConfig) -> IntegrationTarget {
        match config.integration.mode {
            IntegrationMode::Tmux => {
                if config.integration.tmux.enabled {
                    IntegrationTarget::Tmux
                } else {
                    IntegrationTarget::Plain
                }
            }
            IntegrationMode::Shell => self
                .shell
                .clone()
                .map(IntegrationTarget::Shell)
                .unwrap_or(IntegrationTarget::Plain),
            IntegrationMode::Terminal => self
                .terminal
                .clone()
                .map(IntegrationTarget::Terminal)
                .unwrap_or(IntegrationTarget::Plain),
            IntegrationMode::Auto => self.auto_target(config),
        }
    }

    fn auto_target(&self, config: &AppConfig) -> IntegrationTarget {
        if self.inside_tmux && config.integration.tmux.enabled {
            return IntegrationTarget::Tmux;
        }
        if config.integration.shell.enabled {
            if let Some(shell) = self.shell.clone() {
                return IntegrationTarget::Shell(shell);
            }
        }
        if config.integration.terminal.enabled {
            if let Some(terminal) = self.terminal.clone() {
                return IntegrationTarget::Terminal(terminal);
            }
        }
        IntegrationTarget::Plain
    }
}

fn detect_shell() -> Option<ShellKind> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(shell.as_str())
        .to_ascii_lowercase();

    Some(match name.as_str() {
        "bash" => ShellKind::Bash,
        "zsh" => ShellKind::Zsh,
        "fish" => ShellKind::Fish,
        other => ShellKind::Unknown(other.to_string()),
    })
}

fn detect_terminal() -> Option<TerminalKind> {
    if env::var_os("KITTY_PID").is_some() {
        return Some(TerminalKind::Kitty);
    }
    if env::var_os("WEZTERM_EXECUTABLE").is_some() {
        return Some(TerminalKind::WezTerm);
    }
    if env::var("TERM_PROGRAM").ok().as_deref() == Some("iTerm.app") {
        return Some(TerminalKind::Iterm2);
    }
    env::var("TERM_PROGRAM")
        .ok()
        .map(|value| TerminalKind::Unknown(value.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn auto_prefers_tmux_when_present() {
        let env = EnvironmentInfo {
            inside_tmux: true,
            over_ssh: false,
            shell: Some(ShellKind::Zsh),
            terminal: Some(TerminalKind::Kitty),
        };
        let config = AppConfig::default();
        assert_eq!(env.choose_target(&config), IntegrationTarget::Tmux);
    }

    #[test]
    fn auto_falls_back_to_shell_when_not_in_tmux() {
        let env = EnvironmentInfo {
            inside_tmux: false,
            over_ssh: false,
            shell: Some(ShellKind::Bash),
            terminal: Some(TerminalKind::Kitty),
        };
        let config = AppConfig::default();
        assert_eq!(
            env.choose_target(&config),
            IntegrationTarget::Shell(ShellKind::Bash)
        );
    }
}
