use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    core::pack::{Entry, EntryType, Pack, PackMeta},
    paths,
};

const MAX_SOURCE_DEPTH: usize = 16;

pub fn load_local_pack() -> Result<Option<Pack>> {
    let roots = discover_tmux_config_roots();
    if roots.is_empty() {
        return Ok(None);
    }

    let mut visited = HashSet::new();
    let mut entries = Vec::new();
    let mut state = ParseState::default();

    for root in roots {
        load_file(&root, &mut visited, &mut entries, &mut state, 0)?;
    }

    dedupe_entries(&mut entries);

    if entries.is_empty() {
        return Ok(None);
    }

    Ok(Some(Pack {
        pack: PackMeta {
            id: "tmux-local".to_string(),
            tool: "tmux".to_string(),
            title: "tmux Local Bindings".to_string(),
            version: "0.1.0".to_string(),
            source: "local-config".to_string(),
        },
        entries,
    }))
}

fn dedupe_entries(entries: &mut Vec<Entry>) {
    let mut seen = HashSet::new();
    entries.retain(|entry| {
        seen.insert((
            entry.title.clone(),
            entry.keys.clone().unwrap_or_default(),
            entry.command.clone().unwrap_or_default(),
        ))
    });
}

fn discover_tmux_config_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(path) = std::env::var_os("BINDFINDER_TMUX_CONFIG")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        roots.push(path);
    }

    if let Some(home) = paths::home_dir() {
        roots.push(home.join(".tmux.conf"));
    }

    if let Some(config_root) = paths::config_root() {
        roots.push(config_root.join("tmux").join("tmux.conf"));
    }

    roots
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
struct ParseState {
    prefix: String,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            prefix: "Ctrl-b".to_string(),
        }
    }
}

fn load_file(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    entries: &mut Vec<Entry>,
    state: &mut ParseState,
    depth: usize,
) -> Result<()> {
    if depth > MAX_SOURCE_DEPTH {
        return Ok(());
    }

    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }

    let content = fs::read_to_string(&canonical)
        .with_context(|| format!("failed to read tmux config {}", canonical.display()))?;

    for (line_index, line) in content.lines().enumerate() {
        let without_comments = strip_comments(line);
        let trimmed = without_comments.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tokens = tokenize(trimmed);
        if tokens.is_empty() {
            continue;
        }

        match tokens[0].as_str() {
            "source-file" | "source" => {
                if let Some(source_path) = parse_source_target(&tokens, &canonical) {
                    load_file(&source_path, visited, entries, state, depth + 1)?;
                }
            }
            "bind" | "bind-key" => {
                if let Some(entry) =
                    parse_binding(&tokens, &canonical, line_index + 1, entries.len(), state)
                {
                    entries.push(entry);
                }
            }
            "set" | "set-option" => update_prefix(&tokens, state),
            _ => {}
        }
    }

    Ok(())
}

fn parse_binding(
    tokens: &[String],
    source: &Path,
    line_number: usize,
    entry_index: usize,
    state: &ParseState,
) -> Option<Entry> {
    let mut index = 1usize;
    let mut no_prefix = false;
    let mut table: Option<String> = None;

    while index < tokens.len() {
        let token = &tokens[index];
        if !is_tmux_bind_option(token) {
            break;
        }

        match token.as_str() {
            "-n" => no_prefix = true,
            "-r" => {}
            "-T" | "-t" | "-N" => {
                index += 1;
                let value = tokens.get(index)?.clone();
                if matches!(token.as_str(), "-T" | "-t") {
                    table = Some(value);
                }
            }
            flag => {
                if flag.contains('n') {
                    no_prefix = true;
                }
            }
        }
        index += 1;
    }

    let key = tokens.get(index)?.clone();
    let action_tokens = tokens.get(index + 1..)?;
    if action_tokens.is_empty() {
        return None;
    }

    let action = action_tokens.join(" ");
    let display_key = if no_prefix {
        normalize_tmux_key(&key)
    } else {
        format!("{} + {}", state.prefix, normalize_tmux_key(&key))
    };
    let title = describe_action(&action, table.as_deref());
    let mut tags = vec!["local".to_string(), "tmux".to_string()];
    tags.extend(action_tags(&action));
    if let Some(table) = table.as_ref() {
        tags.push(format!("table:{table}"));
    }

    Some(Entry {
        id: format!("local-binding-{}", entry_index + 1),
        entry_type: EntryType::Binding,
        title,
        keys: Some(display_key),
        command: Some(action.clone()),
        description: format!(
            "Local tmux binding from {}:{}",
            source.display(),
            line_number
        ),
        examples: Vec::new(),
        tags,
        aliases: action_aliases(&action),
    })
}

fn parse_source_target(tokens: &[String], source: &Path) -> Option<PathBuf> {
    let mut index = 1usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.starts_with('-') {
            index += 1;
            continue;
        }

        return Some(resolve_tmux_path(token, source));
    }

    None
}

fn is_tmux_bind_option(token: &str) -> bool {
    matches!(token, "-n" | "-r" | "-T" | "-t" | "-N")
        || (token.starts_with('-')
            && token.len() > 2
            && token[1..].chars().all(|ch| matches!(ch, 'n' | 'r')))
}

fn update_prefix(tokens: &[String], state: &mut ParseState) {
    if tokens.is_empty() {
        return;
    }

    let mut index = 1usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token == "prefix" {
            if let Some(value) = tokens.get(index + 1) {
                state.prefix = normalize_tmux_key(value);
            }
            return;
        }
        index += 1;
    }
}

fn resolve_tmux_path(value: &str, source: &Path) -> PathBuf {
    if let Some(stripped) = value.strip_prefix("~/") {
        if let Some(home) = paths::home_dir() {
            return home.join(stripped);
        }
    }

    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        source
            .parent()
            .map(|parent| parent.join(path.clone()))
            .unwrap_or(path)
    }
}

fn strip_comments(line: &str) -> String {
    let mut output = String::new();
    let mut quote: Option<char> = None;

    for ch in line.chars() {
        match quote {
            Some(q) if ch == q => {
                quote = None;
                output.push(ch);
            }
            Some(_) => output.push(ch),
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                output.push(ch);
            }
            None if ch == '#' => break,
            None => output.push(ch),
        }
    }

    output
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in input.chars() {
        match quote {
            Some(q) if ch == q => quote = None,
            Some(_) => current.push(ch),
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn normalize_tmux_key(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    trimmed
        .split('-')
        .map(|part| match part {
            "C" => "Ctrl".to_string(),
            "M" => "Alt".to_string(),
            "S" => "Shift".to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn describe_action(action: &str, table: Option<&str>) -> String {
    let command = action.split_whitespace().next().unwrap_or(action);
    let title = match command {
        "split-window" => "Split pane",
        "new-window" => "New window",
        "copy-mode" => "Copy mode",
        "select-pane" => "Select pane",
        "resize-pane" => "Resize pane",
        "select-window" => "Select window",
        "kill-pane" => "Kill pane",
        "kill-window" => "Kill window",
        "display-popup" => "Display popup",
        _ => command,
    };

    if let Some(table) = table {
        format!("{title} ({table})")
    } else {
        title.to_string()
    }
}

fn action_tags(action: &str) -> Vec<String> {
    let command = action.split_whitespace().next().unwrap_or(action);
    match command {
        "split-window" => vec!["split".into(), "pane".into(), "layout".into()],
        "new-window" => vec!["window".into()],
        "copy-mode" => vec!["copy".into(), "scrollback".into()],
        "select-pane" => vec!["pane".into(), "focus".into()],
        "resize-pane" => vec!["pane".into(), "resize".into()],
        "select-window" => vec!["window".into(), "focus".into()],
        "kill-pane" => vec!["pane".into(), "close".into()],
        "kill-window" => vec!["window".into(), "close".into()],
        "display-popup" => vec!["popup".into()],
        _ => command
            .split('-')
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect(),
    }
}

fn action_aliases(action: &str) -> Vec<String> {
    let command = action.split_whitespace().next().unwrap_or(action);
    match command {
        "split-window" => vec!["split pane".into(), "split window".into()],
        "new-window" => vec!["create window".into()],
        "copy-mode" => vec!["scroll mode".into(), "history mode".into()],
        "select-pane" => vec!["move pane focus".into()],
        "resize-pane" => vec!["resize split".into()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn tokenize_respects_quotes() {
        assert_eq!(
            tokenize("bind-key ']' split-window -h"),
            vec!["bind-key", "]", "split-window", "-h"]
        );
    }

    #[test]
    fn parse_binding_builds_local_entry() {
        let entry = parse_binding(
            &tokenize("bind-key -T prefix ] split-window -h"),
            Path::new("/tmp/tmux.conf"),
            3,
            0,
            &ParseState::default(),
        )
        .expect("binding should parse");

        assert_eq!(entry.title, "Split pane (prefix)");
        assert_eq!(entry.keys.as_deref(), Some("Ctrl-b + ]"));
        assert_eq!(entry.command.as_deref(), Some("split-window -h"));
    }

    #[test]
    fn resolve_source_path_expands_home() {
        let home = paths::home_dir().expect("home should exist in tests");
        let path = resolve_tmux_path("~/test.conf", Path::new("/tmp/tmux.conf"));
        assert_eq!(path, home.join("test.conf"));
    }

    #[test]
    fn load_file_follows_source_file() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bindfinder-tmux-{stamp}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");

        let root = dir.join("tmux.conf");
        let extra = dir.join("extra.conf");
        fs::write(&root, format!("source-file {}\n", extra.display()))
            .expect("root config should be written");
        fs::write(&extra, "bind-key ] split-window -h\n").expect("extra config should be written");

        let mut visited = HashSet::new();
        let mut entries = Vec::new();
        let mut state = ParseState::default();
        load_file(&root, &mut visited, &mut entries, &mut state, 0)
            .expect("tmux config should load");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].keys.as_deref(), Some("Ctrl-b + ]"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dash_key_is_not_treated_as_an_option() {
        let entry = parse_binding(
            &tokenize("bind - split-window -v -c '#{pane_current_path}'"),
            Path::new("/tmp/tmux.conf"),
            1,
            0,
            &ParseState::default(),
        )
        .expect("binding should parse");

        assert_eq!(entry.keys.as_deref(), Some("Ctrl-b + -"));
        assert_eq!(
            entry.command.as_deref(),
            Some("split-window -v -c #{pane_current_path}")
        );
    }

    #[test]
    fn prefix_setting_changes_displayed_prefix() {
        let mut state = ParseState::default();
        update_prefix(&tokenize("set -g prefix C-a"), &mut state);
        let entry = parse_binding(
            &tokenize("bind ] split-window -h"),
            Path::new("/tmp/tmux.conf"),
            1,
            0,
            &state,
        )
        .expect("binding should parse");

        assert_eq!(entry.keys.as_deref(), Some("Ctrl-a + ]"));
    }

    #[test]
    fn modifier_names_are_normalized() {
        assert_eq!(normalize_tmux_key("S-Left"), "Shift-Left");
        assert_eq!(normalize_tmux_key("C-]"), "Ctrl-]");
        assert_eq!(normalize_tmux_key("M-/"), "Alt-/");
    }

    #[test]
    fn dedupe_entries_removes_exact_duplicate_bindings() {
        let mut entries = vec![
            Entry {
                id: "one".into(),
                entry_type: EntryType::Binding,
                title: "Previous window".into(),
                keys: Some("Shift-Left".into()),
                command: Some("previous-window".into()),
                description: "one".into(),
                examples: vec![],
                tags: vec![],
                aliases: vec![],
            },
            Entry {
                id: "two".into(),
                entry_type: EntryType::Binding,
                title: "Previous window".into(),
                keys: Some("Shift-Left".into()),
                command: Some("previous-window".into()),
                description: "two".into(),
                examples: vec![],
                tags: vec![],
                aliases: vec![],
            },
        ];

        dedupe_entries(&mut entries);
        assert_eq!(entries.len(), 1);
    }
}
