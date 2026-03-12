use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};

use crate::core::pack::{Entry, EntryType, Pack, PackMeta};

pub const FEATURED_REPOS: &str = include_str!("../../assets/navi/featured_repos.txt");

pub fn featured_repos() -> Vec<&'static str> {
    FEATURED_REPOS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

pub fn discover_cheat_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_cheat_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn load_repo(root: &Path) -> Result<Vec<Pack>> {
    let repo_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("invalid repo path: {}", root.display()))?;

    discover_cheat_files(root)?
        .into_iter()
        .map(|path| parse_cheat_file(root, repo_name, &path))
        .collect()
}

pub fn parse_cheat_file(root: &Path, repo_name: &str, path: &Path) -> Result<Pack> {
    let content = fs::read_to_string(path)
        .map_err(|err| anyhow!("failed to read {}: {err}", path.display()))?;
    parse_cheat_str(root, repo_name, path, &content)
}

pub fn parse_cheat_str(root: &Path, repo_name: &str, path: &Path, input: &str) -> Result<Pack> {
    let rel = path
        .strip_prefix(root)
        .map_err(|err| anyhow!("failed to strip prefix for {}: {err}", path.display()))?;
    let tool = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| anyhow!("invalid cheat file name: {}", path.display()))?;

    let sectioned_entries = parse_entries(input)?;
    if sectioned_entries.is_empty() {
        bail!("no entries found in {}", path.display());
    }

    let pack_id = format!(
        "navi:{}:{}",
        sanitize_id(repo_name),
        sanitize_id(&rel.display().to_string())
    );
    let pack_title = format!("{} ({})", tool, repo_name);
    let entries = sectioned_entries
        .into_iter()
        .enumerate()
        .map(|(index, raw)| Entry {
            id: format!("{}-{}", sanitize_id(&raw.title), index + 1),
            entry_type: EntryType::Command,
            title: raw.title,
            keys: None,
            command: Some(raw.command),
            description: raw.description,
            examples: Vec::new(),
            tags: raw.tags,
            aliases: raw.aliases,
        })
        .collect::<Vec<_>>();

    let pack = Pack {
        pack: PackMeta {
            id: pack_id,
            tool: tool.to_string(),
            title: pack_title,
            version: "0.1.0".to_string(),
            source: format!("navi-repo:{}", repo_name),
        },
        entries,
    };
    pack.validate()?;
    Ok(pack)
}

#[derive(Debug)]
struct RawEntry {
    title: String,
    description: String,
    command: String,
    tags: Vec<String>,
    aliases: Vec<String>,
}

fn parse_entries(input: &str) -> Result<Vec<RawEntry>> {
    let mut section = None::<String>;
    let mut current_title = None::<String>;
    let mut command_lines = Vec::<String>::new();
    let mut entries = Vec::new();

    let flush = |entries: &mut Vec<RawEntry>,
                 section: &Option<String>,
                 current_title: &mut Option<String>,
                 command_lines: &mut Vec<String>| {
        if let Some(title) = current_title.take() {
            let command = command_lines
                .iter()
                .map(|line| line.trim_end().to_string())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            if !command.is_empty() {
                let mut tags = Vec::new();
                let mut aliases = Vec::new();
                let description = if let Some(section_name) = section {
                    tags.push(sanitize_id(section_name));
                    aliases.push(section_name.clone());
                    format!("{title} ({section_name})")
                } else {
                    title.clone()
                };
                entries.push(RawEntry {
                    title,
                    description,
                    command,
                    tags,
                    aliases,
                });
            }
            command_lines.clear();
        }
    };

    for raw_line in input.lines() {
        let line = raw_line.trim_end();
        if let Some(rest) = line.strip_prefix('%') {
            flush(
                &mut entries,
                &section,
                &mut current_title,
                &mut command_lines,
            );
            section = Some(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            flush(
                &mut entries,
                &section,
                &mut current_title,
                &mut command_lines,
            );
            let title = rest.trim();
            if !title.is_empty() {
                current_title = Some(title.to_string());
            }
            continue;
        }
        if line.starts_with('$') {
            continue;
        }
        if current_title.is_some() {
            if line.trim().is_empty() {
                flush(
                    &mut entries,
                    &section,
                    &mut current_title,
                    &mut command_lines,
                );
            } else {
                command_lines.push(line.to_string());
            }
        }
    }

    flush(
        &mut entries,
        &section,
        &mut current_title,
        &mut command_lines,
    );
    Ok(entries)
}

fn walk_cheat_files(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(dir).map_err(|err| anyhow!("failed to read {}: {err}", dir.display()))?
    {
        let path = entry
            .map_err(|err| anyhow!("failed to read dir entry: {err}"))?
            .path();
        if path.is_dir() {
            walk_cheat_files(&path, acc)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "cheat")
        {
            acc.push(path);
        }
    }
    Ok(())
}

fn sanitize_id(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_cheat_file() {
        let input = r#"
% Git

# Set global git user name
git config --global user.name <name>

# Clone a git repository
git clone -b <branch_name> <repository> <clone_directory>
"#;

        let pack = parse_cheat_str(
            Path::new("/tmp/repo"),
            "denisidoro-cheats",
            Path::new("/tmp/repo/code/git.cheat"),
            input,
        )
        .expect("should parse cheat file");

        assert_eq!(pack.pack.tool, "git");
        assert_eq!(pack.entries.len(), 2);
        assert_eq!(pack.entries[0].title, "Set global git user name");
        assert_eq!(
            pack.entries[0].command.as_deref(),
            Some("git config --global user.name <name>")
        );
    }

    #[test]
    fn exposes_featured_repo_list() {
        let repos = featured_repos();
        assert!(repos.contains(&"denisidoro/cheats"));
    }
}
