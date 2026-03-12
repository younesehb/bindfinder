use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigFile {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub keybindings: KeyBindingsFile,
    #[serde(default)]
    pub integration: IntegrationConfigFile,
}

#[derive(Debug, Clone, Default)]
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
    #[serde(default)]
    pub page_up: Vec<String>,
    #[serde(default)]
    pub page_down: Vec<String>,
    #[serde(default)]
    pub goto_top: Vec<String>,
    #[serde(default)]
    pub goto_bottom: Vec<String>,
    #[serde(default)]
    pub select: Vec<String>,
    #[serde(default)]
    pub search_mode: Vec<String>,
    #[serde(default)]
    pub favorite_entry: Vec<String>,
    #[serde(default)]
    pub hide_entry: Vec<String>,
    #[serde(default)]
    pub favorite_tool: Vec<String>,
    #[serde(default)]
    pub hide_tool: Vec<String>,
    #[serde(default)]
    pub toggle_hidden: Vec<String>,
    #[serde(default)]
    pub toggle_favorites_only: Vec<String>,
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
    pub page_up: Vec<KeyBinding>,
    pub page_down: Vec<KeyBinding>,
    pub goto_top: Vec<KeySequence>,
    pub goto_bottom: Vec<KeySequence>,
    pub select: Vec<KeyBinding>,
    pub search_mode: Vec<KeyBinding>,
    pub favorite_entry: Vec<KeyBinding>,
    pub hide_entry: Vec<KeyBinding>,
    pub favorite_tool: Vec<KeyBinding>,
    pub hide_tool: Vec<KeyBinding>,
    pub toggle_hidden: Vec<KeyBinding>,
    pub toggle_favorites_only: Vec<KeyBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySequence {
    pub steps: Vec<KeyBinding>,
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
    #[serde(default)]
    pub use_popup: bool,
    #[serde(default = "default_popup_width")]
    pub popup_width: String,
    #[serde(default = "default_popup_height")]
    pub popup_height: String,
    #[serde(default)]
    pub debug: bool,
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

    pub fn load_with_path() -> Result<(Self, Option<PathBuf>)> {
        let path = default_config_path();
        let config = Self::load_from_path(path.as_deref())?;
        Ok((config, path))
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
        validate_shell_binding_syntax(&self.integration.shell.binding)?;
        validate_tmux_key_syntax(&self.integration.tmux.key)?;
        Ok(())
    }

    pub fn to_yaml_string(&self) -> Result<String> {
        let file = AppConfigFile::from(self);
        Ok(serde_yaml::to_string(&file)?)
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
            quit: parse_bindings(["q", "esc", "ctrl-c"]).expect("default quit bindings"),
            clear_query: parse_bindings(["ctrl-u"]).expect("default clear bindings"),
            move_up: parse_bindings(["up", "k"]).expect("default move up bindings"),
            move_down: parse_bindings(["down", "j"]).expect("default move down bindings"),
            page_up: parse_bindings(["pageup", "ctrl-u"]).expect("default page up bindings"),
            page_down: parse_bindings(["pagedown", "ctrl-d"]).expect("default page down bindings"),
            goto_top: parse_sequences(["g g"]).expect("default goto top bindings"),
            goto_bottom: parse_sequences(["shift-g"]).expect("default goto bottom bindings"),
            select: parse_bindings(["enter"]).expect("default select bindings"),
            search_mode: parse_bindings(["/"]).expect("default search mode bindings"),
            favorite_entry: parse_bindings(["f"]).expect("default favorite entry bindings"),
            hide_entry: parse_bindings(["x"]).expect("default hide entry bindings"),
            favorite_tool: parse_bindings(["shift-f"]).expect("default favorite tool bindings"),
            hide_tool: parse_bindings(["shift-x"]).expect("default hide tool bindings"),
            toggle_hidden: parse_bindings(["z"]).expect("default toggle hidden bindings"),
            toggle_favorites_only: parse_bindings(["m"])
                .expect("default toggle favorites bindings"),
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
            use_popup: false,
            popup_width: default_popup_width(),
            popup_height: default_popup_height(),
            debug: false,
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

    pub fn matches_page_up(&self, key: KeyEvent) -> bool {
        self.page_up.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_page_down(&self, key: KeyEvent) -> bool {
        self.page_down.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_select(&self, key: KeyEvent) -> bool {
        self.select.iter().any(|binding| binding.matches(key))
    }

    pub fn key_from_event(&self, key: KeyEvent) -> KeyBinding {
        normalize_event_key(key)
    }

    pub fn matches_search_mode(&self, key: KeyEvent) -> bool {
        self.search_mode.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_favorite_entry(&self, key: KeyEvent) -> bool {
        self.favorite_entry
            .iter()
            .any(|binding| binding.matches(key))
    }

    pub fn matches_hide_entry(&self, key: KeyEvent) -> bool {
        self.hide_entry.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_favorite_tool(&self, key: KeyEvent) -> bool {
        self.favorite_tool
            .iter()
            .any(|binding| binding.matches(key))
    }

    pub fn matches_hide_tool(&self, key: KeyEvent) -> bool {
        self.hide_tool.iter().any(|binding| binding.matches(key))
    }

    pub fn matches_toggle_hidden(&self, key: KeyEvent) -> bool {
        self.toggle_hidden
            .iter()
            .any(|binding| binding.matches(key))
    }

    pub fn matches_toggle_favorites_only(&self, key: KeyEvent) -> bool {
        self.toggle_favorites_only
            .iter()
            .any(|binding| binding.matches(key))
    }
}

impl KeyBinding {
    pub fn matches(&self, key: KeyEvent) -> bool {
        self == &normalize_event_key(key)
    }
}

impl KeySequence {
    pub fn matches_exact(&self, keys: &[KeyBinding]) -> bool {
        self.steps == keys
    }

    pub fn matches_prefix(&self, keys: &[KeyBinding]) -> bool {
        self.steps.starts_with(keys)
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
            page_up: value.page_up.iter().map(format_binding).collect(),
            page_down: value.page_down.iter().map(format_binding).collect(),
            goto_top: value.goto_top.iter().map(format_sequence).collect(),
            goto_bottom: value.goto_bottom.iter().map(format_sequence).collect(),
            select: value.select.iter().map(format_binding).collect(),
            search_mode: value.search_mode.iter().map(format_binding).collect(),
            favorite_entry: value.favorite_entry.iter().map(format_binding).collect(),
            hide_entry: value.hide_entry.iter().map(format_binding).collect(),
            favorite_tool: value.favorite_tool.iter().map(format_binding).collect(),
            hide_tool: value.hide_tool.iter().map(format_binding).collect(),
            toggle_hidden: value.toggle_hidden.iter().map(format_binding).collect(),
            toggle_favorites_only: value
                .toggle_favorites_only
                .iter()
                .map(format_binding)
                .collect(),
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
            page_up: choose_or_parse(value.page_up, &["pageup", "ctrl-u"])?,
            page_down: choose_or_parse(value.page_down, &["pagedown", "ctrl-d"])?,
            goto_top: choose_or_parse_sequences(value.goto_top, &["g g"])?,
            goto_bottom: choose_or_parse_sequences(value.goto_bottom, &["shift-g"])?,
            select: choose_or_parse(value.select, &["enter"])?,
            search_mode: choose_or_parse(value.search_mode, &["/"])?,
            favorite_entry: choose_or_parse(value.favorite_entry, &["f"])?,
            hide_entry: choose_or_parse(value.hide_entry, &["x"])?,
            favorite_tool: choose_or_parse(value.favorite_tool, &["shift-f"])?,
            hide_tool: choose_or_parse(value.hide_tool, &["shift-x"])?,
            toggle_hidden: choose_or_parse(value.toggle_hidden, &["z"])?,
            toggle_favorites_only: choose_or_parse(value.toggle_favorites_only, &["m"])?,
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

fn choose_or_parse_sequences(values: Vec<String>, defaults: &[&str]) -> Result<Vec<KeySequence>> {
    if values.is_empty() {
        parse_sequences(defaults)
    } else {
        parse_sequences(values.iter().map(String::as_str))
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

fn parse_sequences<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<KeySequence>>
where
    T: AsRef<str>,
{
    values
        .into_iter()
        .map(|value| parse_sequence(value.as_ref()))
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

fn parse_sequence(value: &str) -> Result<KeySequence> {
    let steps = value
        .split_whitespace()
        .map(parse_binding)
        .collect::<Result<Vec<_>>>()?;
    if steps.is_empty() {
        bail!("empty key sequence is not allowed");
    }
    Ok(KeySequence { steps })
}

fn parse_key_code(value: &str) -> Result<KeyCode> {
    match value {
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "pageup" => Ok(KeyCode::PageUp),
        "pagedown" => Ok(KeyCode::PageDown),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "enter" => Ok(KeyCode::Enter),
        "tab" => Ok(KeyCode::Tab),
        "backspace" => Ok(KeyCode::Backspace),
        "esc" | "escape" => Ok(KeyCode::Esc),
        single if single.chars().count() == 1 => {
            Ok(KeyCode::Char(single.chars().next().expect("single char")))
        }
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

    paths::bindfinder_config_file("config.yaml")
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
        KeyCode::PageUp => "pageup".to_string(),
        KeyCode::PageDown => "pagedown".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
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

fn format_sequence(sequence: &KeySequence) -> String {
    sequence
        .steps
        .iter()
        .map(format_binding)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_event_key(key: KeyEvent) -> KeyBinding {
    let code = match key.code {
        KeyCode::Char(ch) => KeyCode::Char(normalize_char_code(ch, key.modifiers)),
        other => other,
    };

    KeyBinding {
        code,
        modifiers: key.modifiers,
    }
}

fn normalize_char_code(ch: char, modifiers: KeyModifiers) -> char {
    if modifiers.contains(KeyModifiers::SHIFT) {
        normalize_shifted_char(ch)
    } else {
        ch
    }
}

fn normalize_shifted_char(ch: char) -> char {
    if ch.is_ascii_uppercase() {
        return ch.to_ascii_lowercase();
    }

    match ch {
        '!' => '1',
        '@' => '2',
        '#' => '3',
        '$' => '4',
        '%' => '5',
        '^' => '6',
        '&' => '7',
        '*' => '8',
        '(' => '9',
        ')' => '0',
        '_' => '-',
        '+' => '=',
        '{' => '[',
        '}' => ']',
        '|' => '\\',
        ':' => ';',
        '"' => '\'',
        '<' => ',',
        '>' => '.',
        '?' => '/',
        '~' => '`',
        other => other,
    }
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
    "ctrl-]".to_string()
}

fn default_tmux_key() -> String {
    "C-]".to_string()
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

fn validate_shell_binding_syntax(value: &str) -> Result<()> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(());
    }

    if normalized.starts_with("c-") {
        bail!(
            "integration.shell.binding uses shell syntax like ctrl-] or alt-/, not tmux syntax like C-]"
        );
    }

    Ok(())
}

fn validate_tmux_key_syntax(value: &str) -> Result<()> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(());
    }

    if normalized.starts_with("ctrl-")
        || normalized.starts_with("alt-")
        || normalized.starts_with("shift-")
    {
        bail!("integration.tmux.key uses tmux syntax like C-] or /, not shell syntax like ctrl-]");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_config_has_expected_bindings() {
        let config = AppConfig::default();
        assert_eq!(config.settings.result_list_width_percent, 45);
        assert!(config
            .keybindings
            .matches_move_up(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)));
        assert!(config
            .keybindings
            .matches_quit(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
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
  goto_top: ["g g"]
  goto_bottom: ["shift-g"]
  hide_entry: ["o"]
integration:
  mode: "tmux"
  launch_key: "ctrl-]"
  tmux:
    key: "?"
    use_popup: false
    debug: true
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
        assert_eq!(config.integration.launch_key, "ctrl-]");
        assert_eq!(config.integration.tmux.key, "?");
        assert!(!config.integration.tmux.use_popup);
        assert!(config.integration.tmux.debug);
        assert!(config
            .keybindings
            .matches_move_up(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE)));
        assert!(config
            .keybindings
            .matches_hide_entry(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)));
        assert!(config
            .keybindings
            .matches_quit(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)));
    }

    #[test]
    fn default_config_serializes_with_integration_block() {
        let yaml = AppConfig::default()
            .to_yaml_string()
            .expect("default config should serialize");
        assert!(yaml.contains("integration:"));
        assert!(yaml.contains("launch_key: ctrl-]"));
    }

    #[test]
    fn shifted_char_events_match_lowercase_shift_bindings() {
        let bindings = KeyBindings::default();
        let upper_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert_eq!(
            bindings.key_from_event(upper_g),
            KeyBinding {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::SHIFT,
            }
        );
        assert!(bindings
            .goto_bottom
            .iter()
            .any(|sequence| sequence.matches_exact(&[bindings.key_from_event(upper_g)])));
    }

    #[test]
    fn parser_supports_home_and_end_keys() {
        let home = parse_binding("home").expect("home should parse");
        let end = parse_binding("end").expect("end should parse");
        assert_eq!(home.code, KeyCode::Home);
        assert_eq!(end.code, KeyCode::End);
        assert_eq!(format_binding(&home), "home");
        assert_eq!(format_binding(&end), "end");
    }

    #[test]
    fn shell_binding_rejects_tmux_style_key_notation() {
        let yaml = r#"
integration:
  shell:
    binding: "C-]"
"#;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = env::temp_dir().join(format!("bindfinder-config-{stamp}.yaml"));
        fs::write(&path, yaml).expect("write config");

        let err = AppConfig::load_from_path(Some(&path)).expect_err("config should fail");
        fs::remove_file(&path).ok();

        assert!(err
            .to_string()
            .contains("integration.shell.binding uses shell syntax like ctrl-]"));
    }

    #[test]
    fn tmux_key_rejects_shell_style_key_notation() {
        let yaml = r#"
integration:
  tmux:
    key: "ctrl-]"
"#;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = env::temp_dir().join(format!("bindfinder-config-{stamp}.yaml"));
        fs::write(&path, yaml).expect("write config");

        let err = AppConfig::load_from_path(Some(&path)).expect_err("config should fail");
        fs::remove_file(&path).ok();

        assert!(err
            .to_string()
            .contains("integration.tmux.key uses tmux syntax like C-]"));
    }
}
