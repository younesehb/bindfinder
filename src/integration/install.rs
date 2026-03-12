use crate::{
    config::AppConfig,
    integration::detect::{EnvironmentInfo, IntegrationTarget, ShellKind, TerminalKind},
};

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
    if config.integration.tmux.use_popup {
        format!(
            "bind-key {} display-popup -w {} -h {} -E 'bindfinder'",
            config.integration.tmux.key,
            config.integration.tmux.popup_width,
            config.integration.tmux.popup_height
        )
    } else {
        format!(
            "bind-key {} split-window -v 'bindfinder'",
            config.integration.tmux.key
        )
    }
}

fn render_shell(config: &AppConfig, shell: &ShellKind) -> String {
    match shell {
        ShellKind::Bash => format!(
            "bind -x '\"\\\\C-_\":\"bindfinder\"'\n# requested launch key: {}",
            config.integration.shell.binding
        ),
        ShellKind::Zsh => format!(
            "bindkey '^_' bindfinder-widget\nbindfinder-widget() {{ bindfinder }}\nzle -N bindfinder-widget\n# requested launch key: {}",
            config.integration.shell.binding
        ),
        ShellKind::Fish => format!(
            "function bindfinder_widget\n    bindfinder\nend\nbind {} bindfinder_widget",
            fish_binding(&config.integration.shell.binding)
        ),
        ShellKind::Unknown(name) => format!(
            "# shell '{}' is not directly supported yet\nbindfinder",
            name
        ),
    }
}

fn render_terminal(config: &AppConfig, terminal: &TerminalKind) -> String {
    match terminal {
        TerminalKind::WezTerm => format!(
            "keys = {{ {{ key = '/', mods = 'CTRL', action = wezterm.action.SpawnCommandInNewTab {{ args = {{ 'bindfinder' }} }} }} }} -- requested launch key: {}",
            config.integration.launch_key
        ),
        TerminalKind::Kitty => format!(
            "map ctrl+/ launch --type=overlay bindfinder\n# requested launch key: {}",
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

fn fish_binding(binding: &str) -> &str {
    if binding == "ctrl-/" {
        "\\c/"
    } else {
        "\\cf"
    }
}
