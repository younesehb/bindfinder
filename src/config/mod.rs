use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigFile {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub keybindings: KeyBindingsFile,
    #[serde(default)]
    pub integration: IntegrationConfigFile,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub settings: Settings,
    pub keybindings: KeyBindings,
    pub integration: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_result_list_width_percent")]
    pub result_list_width_percent: u16,
    #[serde(default = "default_show_footer")]
    pub show_footer: bool,
    #[serde(default = "default_wrap_preview")]
    pub wrap_preview: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBindingsFile {
    #[serde(default)]
    pub quit: Vec<String>,
    #[serde(default)]
    pub clear_query: Vec<String>,
    #[serde(default)]
    pub move_up: Vec<String>,
    #[serde(default)]
    pub move_down: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationConfigFile {
    #[serde(default)]
    pub mode: IntegrationMode,
    #[serde(default = "default_launch_key")]
    pub launch_key: String,
    #[serde(default)]
    pub tmux: TmuxConfig,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub quit: Vec<KeyBinding>,
    pub clear_query: Vec<KeyBinding>,
    pub move_up: Vec<KeyBinding>,
    pub move_down: Vec<KeyBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    pub mode: IntegrationMode,
    pub launch_key: String,
    pub tmux: TmuxConfig,
    pub shell: ShellConfig,
    pub terminal: TerminalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationMode {
    #[default]
    Auto,
    Tmux,
    Shell,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_tmux_key")]
    pub key: String,
    #[serde(default = "default_true")]
    pub use_popup: bool,
    #[serde(default = "default_popup_width")]
    pub popup_width: String,
    #[serde(default = "default_popup_height")]
    pub popup_height: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shell_preferred")]
    pub preferred: String,
    #[serde(default = "default_launch_key")]
    pub binding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_terminal_preferred")]
    pub preferred: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = default_config_path();
        Self::load_from_path(path.as_deref())
    }

    pub fn load_from_path(path: Option<&Path>) -> Result<Self> {
        let mut config = Self::default();

        if let Some(path) = path {
            if path.exists() {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                let parsed: AppConfigFile = serde_yaml::from_str(&content)
                    .with_context(|| format!("failed to parse {}", path.display()))?;
                config = parsed.try_into()?;
            }
        }

        config.validate()?;
        Ok(config)
    }

    pub fn default_path() -> Option<PathBuf> {
        default_config_path()
    }

    fn validate(&self) -> Result<()> {
        if !(20..=80).contains(&self.settings.result_list_width_percent) {
            bail!("settings.result_list_width_percent must be between 20 and 80");
        }
        if self.keybindings.quit.is_empty() {
            bail!("keybindings.quit must not be empty");
        }
        if self.integration.launch_key.trim().is_empty() {
            bail!("integration.launch_key must not be empty");
        }
        if self.integration.tmux.key.trim().is_empty() {
            bail!("integration.tmux.key must not be empty");
        }
        if self.integration.tmux.popup_width.trim().is_empty() {
            bail!("integration.tmux.popup_width must not be empty");
        }
        if self.integration.tmux.popup_height.trim().is_empty() {
            bail!("integration.tmux.popup_height must not be empty");
        }
        if self.integration.shell.binding.trim().is_empty() {
            bail!("integration.shell.binding must not be empty");
        }
        Ok(())
    }

    pub fn to_yaml_string(&self) -> Result<String> {
        let file = AppConfigFile::from(self);
        Ok(serde_yaml::to_string(&file)?)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            keybindings: KeyBindings::default(),
            integration: IntegrationConfig::default(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            result_list_width_percent: default_result_list_width_percent(),
            show_footer: default_show_footer(),
            wrap_preview: default_wrap_preview(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: parse_bindings(&["q", "esc", "ctrl-c"]).expect("default quit bindings"),
            clear_query: parse_bindings(&["ctrl-u"]).expect("default clear bindings"),
            move_up: parse_bindings(&["up", "k"]).expect("default move up bindings"),
            move_down: parse_bindings(&["down", "j"]).expect("default move down bindings"),
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            mode: IntegrationMode::Auto,
            launch_key: default_launch_key(),
            tmux: TmuxConfig::default(),
            shell: ShellConfig::default(),
            terminal: TerminalConfig::default(),
        }
    }
}

impl Default for TmuxConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            key: default_tmux_key(),
            use_popup: default_true(),
            popup_width: default_popup_width(),
            popup_height: default_popup_height(),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            preferred: default_shell_preferred(),
            binding: default_launch_key(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preferred: default_terminal_preferred(),
        }
    }
}

impl KeyBindings {
    pub fn matches_quit(&self, key: KeyEvent) -> bool {
        self.quit.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_clear_query(&self, key: KeyEvent) -> bool {
        self.clear_query.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_move_up(&self, key: KeyEvent) -> bool {
        self.move_up.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_move_down(&self, key: KeyEvent) -> bool {
        self.move_down.iter().any(|binding| binding.matches(key))
    }
}

impl KeyBinding {
    pub fn matches(&self, key: KeyEvent) -> bool {
        self.code == key.code && self.modifiers == key.modifiers
    }
}

impl TryFrom<AppConfigFile> for AppConfig {
    type Error = anyhow::Error;

    fn try_from(value: AppConfigFile) -> Result<Self> {
        Ok(Self {
            settings: value.settings,
            keybindings: KeyBindings::try_from(value.keybindings)?,
            integration: IntegrationConfig::from(value.integration),
        })
    }
}

impl From<IntegrationConfigFile> for IntegrationConfig {
    fn from(value: IntegrationConfigFile) -> Self {
        Self {
            mode: value.mode,
            launch_key: value.launch_key,
            tmux: value.tmux,
            shell: value.shell,
            terminal: value.terminal,
        }
    }
}

impl From<&AppConfig> for AppConfigFile {
    fn from(value: &AppConfig) -> Self {
        Self {
            settings: value.settings.clone(),
            keybindings: KeyBindingsFile::from(&value.keybindings),
            integration: IntegrationConfigFile::from(&value.integration),
        }
    }
}

impl From<&KeyBindings> for KeyBindingsFile {
    fn from(value: &KeyBindings) -> Self {
        Self {
            quit: value.quit.iter().map(format_binding).collect(),
            clear_query: value.clear_query.iter().map(format_binding).collect(),
            move_up: value.move_up.iter().map(format_binding).collect(),
            move_down: value.move_down.iter().map(format_binding).collect(),
        }
    }
}

impl From<&IntegrationConfig> for IntegrationConfigFile {
    fn from(value: &IntegrationConfig) -> Self {
        Self {
            mode: value.mode.clone(),
            launch_key: value.launch_key.clone(),
            tmux: value.tmux.clone(),
            shell: value.shell.clone(),
            terminal: value.terminal.clone(),
        }
    }
}

impl TryFrom<KeyBindingsFile> for KeyBindings {
    type Error = anyhow::Error;

    fn try_from(value: KeyBindingsFile) -> Result<Self> {
        Ok(Self {
            quit: choose_or_parse(value.quit, &["q", "esc", "ctrl-c"])?,
            clear_query: choose_or_parse(value.clear_query, &["ctrl-u"])?,
            move_up: choose_or_parse(value.move_up, &["up", "k"])?,
            move_down: choose_or_parse(value.move_down, &["down", "j"])?,
        })
    }
}

fn choose_or_parse(values: Vec<String>, defaults: &[&str]) -> Result<Vec<KeyBinding>> {
    if values.is_empty() {
        parse_bindings(defaults)
    } else {
        parse_bindings(values.iter().map(String::as_str))
    }
}

fn parse_bindings<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<KeyBinding>>
where
    T: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| parse_binding(value.as_ref()))
        .collect()
}

fn parse_binding(value: &str) -> Result<KeyBinding> {
    let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
    if normalized.is_empty() {
        bail!("empty keybinding is not allowed");
    }

    let mut modifiers = KeyModifiers::NONE;
    let mut key_name = None;

    for segment in normalized.split('-') {
        match segment {
            "ctrl" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            key => key_name = Some(key),
        }
    }

    let key_name = key_name.ok_or_else(|| anyhow!("invalid keybinding: {value}"))?;
    let code = parse_key_code(key_name)?;

    Ok(KeyBinding { code, modifiers })
}

fn parse_key_code(value: &str) -> Result<KeyCode> {
    match value {
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "enter" => Ok(KeyCode::Enter),
        "tab" => Ok(KeyCode::Tab),
        "backspace" => Ok(KeyCode::Backspace),
        "esc" | "escape" => Ok(KeyCode::Esc),
        single if single.chars().count() == 1 => Ok(KeyCode::Char(
            single.chars().next().expect("single char"),
        )),
        _ => bail!("unsupported keybinding key: {value}"),
    }
}

fn default_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("BINDFINDER_CONFIG") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }

    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(dir).join("bindfinder").join("config.yaml"));
    }

    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config").join("bindfinder").join("config.yaml"))
}

fn format_binding(binding: &KeyBinding) -> String {
    let mut parts = Vec::new();
    if binding.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl".to_string());
    }
    if binding.modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt".to_string());
    }
    if binding.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("shift".to_string());
    }

    let key = match binding.code {
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Char(ch) => ch.to_string(),
        _ => "?".to_string(),
    };

    parts.push(key);
    parts.join("-")
}

fn default_result_list_width_percent() -> u16 {
    45
}

fn default_show_footer() -> bool {
    true
}

fn default_wrap_preview() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_launch_key() -> String {
    "ctrl-/".to_string()
}

fn default_tmux_key() -> String {
    "/".to_string()
}

fn default_popup_width() -> String {
    "80%".to_string()
}

fn default_popup_height() -> String {
    "80%".to_string()
}

fn default_shell_preferred() -> String {
    "auto".to_string()
}

fn default_terminal_preferred() -> String {
    "auto".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_config_has_expected_bindings() {
        let config = AppConfig::default();
        assert_eq!(config.settings.result_list_width_percent, 45);
        assert!(config.keybindings.matches_move_up(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE
        )));
        assert!(config.keybindings.matches_quit(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn yaml_config_overrides_bindings_and_settings() {
        let yaml = r#"
settings:
  result_list_width_percent: 50
  show_footer: false
  wrap_preview: false
keybindings:
  quit: ["x"]
  clear_query: ["ctrl-l"]
  move_up: ["w"]
  move_down: ["s"]
integration:
  mode: "tmux"
  launch_key: "ctrl-g"
  tmux:
    key: "?"
    use_popup: false
"#;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = env::temp_dir().join(format!("bindfinder-config-{stamp}.yaml"));
        fs::write(&path, yaml).expect("write config");

        let config = AppConfig::load_from_path(Some(&path)).expect("config should parse");
        fs::remove_file(&path).ok();

        assert_eq!(config.settings.result_list_width_percent, 50);
        assert!(!config.settings.show_footer);
        assert!(!config.settings.wrap_preview);
        assert_eq!(config.integration.mode, IntegrationMode::Tmux);
        assert_eq!(config.integration.launch_key, "ctrl-g");
        assert_eq!(config.integration.tmux.key, "?");
        assert!(!config.integration.tmux.use_popup);
        assert!(config.keybindings.matches_move_up(KeyEvent::new(
            KeyCode::Char('w'),
            KeyModifiers::NONE
        )));
        assert!(config.keybindings.matches_quit(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn default_config_serializes_with_integration_block() {
        let yaml = AppConfig::default()
            .to_yaml_string()
            .expect("default config should serialize");
        assert!(yaml.contains("integration:"));
        assert!(yaml.contains("launch_key: ctrl-/"));
    }
}
