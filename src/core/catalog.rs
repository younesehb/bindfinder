use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::core::{
    navi,
    pack::{Entry, Pack, parse_pack_file, parse_pack_str},
};
use crate::paths;
use crate::state::UserState;

#[derive(Debug, Clone)]
pub struct Catalog {
    entries: Vec<CatalogEntry>,
}

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub pack_id: String,
    pub tool: String,
    pub pack_title: String,
    pub source: String,
    pub entry: Entry,
}

impl Catalog {
    pub fn load_all() -> Result<Self> {
        let mut packs = [include_str!("../../assets/packs/tmux.yaml")]
            .into_iter()
            .map(parse_pack_str)
            .collect::<Result<Vec<_>>>()?;

        if let Some(dir) = default_pack_dir() {
            for path in discover_pack_files(&dir)? {
                packs.push(parse_pack_file(&path)?);
            }
        }
        if let Some(dir) = default_navi_repo_dir() {
            for repo_dir in discover_repo_dirs(&dir)? {
                packs.extend(navi::load_repo(&repo_dir)?);
            }
        }

        Self::from_packs(packs)
    }

    pub fn default_pack_dir() -> Option<PathBuf> {
        default_pack_dir()
    }

    pub fn default_navi_repo_dir() -> Option<PathBuf> {
        default_navi_repo_dir()
    }

    pub fn from_packs(packs: Vec<Pack>) -> Result<Self> {
        let mut seen_pack_ids = std::collections::HashSet::new();
        let mut seen_entries = std::collections::HashSet::new();
        let mut entries = Vec::new();

        for pack in packs {
            if !seen_pack_ids.insert(pack.pack.id.clone()) {
                return Err(anyhow!("duplicate pack id: {}", pack.pack.id));
            }

            let pack_id = pack.pack.id.clone();
            let tool = pack.pack.tool.clone();
            let pack_title = pack.pack.title.clone();
            let source = pack.pack.source.clone();

            for entry in pack.entries {
                let qualified_id = format!("{pack_id}:{}", entry.id);
                if !seen_entries.insert(qualified_id.clone()) {
                    return Err(anyhow!("duplicate qualified entry id: {qualified_id}"));
                }

                entries.push(CatalogEntry {
                    pack_id: pack_id.clone(),
                    tool: tool.clone(),
                    pack_title: pack_title.clone(),
                    source: source.clone(),
                    entry,
                });
            }
        }

        Ok(Self { entries })
    }

    pub fn filter_with_state<'a>(
        &'a self,
        query: &str,
        state: &UserState,
        include_hidden: bool,
        favorites_only: bool,
    ) -> Vec<&'a CatalogEntry> {
        let query = query.trim().to_ascii_lowercase();
        let terms = query
            .split_whitespace()
            .filter(|term| !term.is_empty())
            .collect::<Vec<_>>();
        let mut matches = self
            .entries
            .iter()
            .filter(|item| {
                include_hidden
                    || (!state.is_tool_hidden(&item.tool) && !state.is_entry_hidden(&item.qualified_id()))
            })
            .filter(|item| {
                !favorites_only
                    || state.is_entry_favorite(&item.qualified_id())
                    || state.is_tool_favorite(&item.tool)
            })
            .filter(|item| matches_query(item, &query, &terms))
            .collect::<Vec<_>>();

        matches.sort_by_key(|item| {
            (
                rank(item, &query, &terms, state),
                item.tool.as_str(),
                item.entry.title.as_str(),
            )
        });

        matches
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn tools(&self) -> Vec<&str> {
        let mut tools = self
            .entries
            .iter()
            .map(|item| item.tool.as_str())
            .collect::<Vec<_>>();
        tools.sort_unstable();
        tools.dedup();
        tools
    }
}

impl CatalogEntry {
    pub fn qualified_id(&self) -> String {
        format!("{}:{}", self.pack_id, self.entry.id)
    }
}

fn matches_query(item: &CatalogEntry, query: &str, terms: &[&str]) -> bool {
    if query.is_empty() {
        return true;
    }

    let blob = search_blob(item);
    terms.iter().all(|term| blob.contains(term))
}

fn rank(item: &CatalogEntry, query: &str, terms: &[&str], state: &UserState) -> usize {
    let base: usize = if query.is_empty() {
        100
    } else if terms.len() == 1 && item.tool.eq_ignore_ascii_case(query) {
        0
    } else if item.entry.title.eq_ignore_ascii_case(query) {
        1
    } else if item
        .entry
        .aliases
        .iter()
        .any(|alias| alias.eq_ignore_ascii_case(query))
    {
        2
    } else if item
        .entry
        .command
        .as_ref()
        .is_some_and(|command| command.eq_ignore_ascii_case(query))
    {
        3
    } else if item.tool.eq_ignore_ascii_case(query) {
        4
    } else if item.entry.title.to_ascii_lowercase().contains(query) {
        5
    } else if item
        .entry
        .aliases
        .iter()
        .any(|alias| alias.to_ascii_lowercase().contains(query))
    {
        6
    } else if item
        .entry
        .command
        .as_ref()
        .is_some_and(|command| command.to_ascii_lowercase().contains(query))
    {
        7
    } else if item
        .entry
        .description
        .to_ascii_lowercase()
        .contains(query)
    {
        8
    } else if terms.iter().all(|term| item.tool.to_ascii_lowercase().contains(term)) {
        9
    } else {
        20
    };

    let boost: usize = if state.is_entry_favorite(&item.qualified_id()) {
        30
    } else if state.is_tool_favorite(&item.tool) {
        10
    } else {
        0
    };

    base.saturating_sub(boost)
}

fn search_blob(item: &CatalogEntry) -> String {
    let mut fields = vec![
        item.tool.to_ascii_lowercase(),
        item.pack_title.to_ascii_lowercase(),
        item.entry.title.to_ascii_lowercase(),
        item.entry.description.to_ascii_lowercase(),
        item.entry.entry_type.as_str().to_string(),
    ];

    if let Some(keys) = &item.entry.keys {
        fields.push(keys.to_ascii_lowercase());
    }

    if let Some(command) = &item.entry.command {
        fields.push(command.to_ascii_lowercase());
    }

    fields.extend(item.entry.tags.iter().map(|tag| tag.to_ascii_lowercase()));
    fields.extend(item.entry.aliases.iter().map(|alias| alias.to_ascii_lowercase()));

    fields.join("\n")
}

fn default_pack_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("BINDFINDER_PACK_DIR") {
        let path = PathBuf::from(dir);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }

    paths::bindfinder_config_dir("packs")
}

fn default_navi_repo_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("BINDFINDER_NAVI_REPOS_DIR") {
        let path = PathBuf::from(dir);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }

    paths::bindfinder_data_dir("repos")
}

fn discover_pack_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = fs::read_dir(dir)
        .map_err(|err| anyhow!("failed to read {}: {err}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "yml" | "yaml"))
        })
        .collect::<Vec<_>>();

    paths.sort();
    Ok(paths)
}

fn discover_repo_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = fs::read_dir(dir)
        .map_err(|err| anyhow!("failed to read {}: {err}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pack::{EntryType, PackMeta};

    fn sample_pack() -> Pack {
        Pack {
            pack: PackMeta {
                id: "sample".into(),
                tool: "tmux".into(),
                title: "Sample".into(),
                version: "0.1.0".into(),
                source: "test".into(),
            },
            entries: vec![
                Entry {
                    id: "split-horizontal".into(),
                    entry_type: EntryType::Binding,
                    title: "Split Pane Horizontally".into(),
                    keys: Some("Prefix + \"".into()),
                    command: Some("split-window".into()),
                    description: "Split the current pane into top and bottom panes.".into(),
                    examples: vec!["Use in pane workflows.".into()],
                    tags: vec!["panes".into()],
                    aliases: vec!["split pane".into()],
                },
                Entry {
                    id: "copy-mode".into(),
                    entry_type: EntryType::Binding,
                    title: "Enter Copy Mode".into(),
                    keys: Some("Prefix + [".into()),
                    command: Some("copy-mode".into()),
                    description: "Enter scrollback and selection mode.".into(),
                    examples: vec![],
                    tags: vec!["scrollback".into()],
                    aliases: vec!["history mode".into()],
                },
            ],
        }
    }

    #[test]
    fn multi_term_filter_matches_expected_entry() {
        let catalog = Catalog::from_packs(vec![sample_pack()]).expect("catalog should build");
        let matches = catalog.filter_with_state("tmux split", &UserState::default(), false, false);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entry.id, "split-horizontal");
    }

    #[test]
    fn tools_are_deduplicated() {
        let catalog = Catalog::from_packs(vec![sample_pack()]).expect("catalog should build");
        assert_eq!(catalog.tools(), vec!["tmux"]);
    }
}
