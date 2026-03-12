use serde::{Deserialize, Serialize};
use std::fmt;

use anyhow::{anyhow, bail, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub pack: PackMeta,
    #[serde(default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMeta {
    pub id: String,
    pub tool: String,
    pub title: String,
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub title: String,
    #[serde(default)]
    pub keys: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    pub description: String,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Binding,
    Command,
    Snippet,
    Workflow,
    Note,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Command => "command",
            Self::Snippet => "snippet",
            Self::Workflow => "workflow",
            Self::Note => "note",
        }
    }
}

impl Pack {
    pub fn validate(&self) -> Result<()> {
        if self.pack.id.trim().is_empty() {
            bail!("pack.id must not be empty");
        }
        if self.pack.tool.trim().is_empty() {
            bail!("pack.tool must not be empty");
        }
        if self.pack.title.trim().is_empty() {
            bail!("pack.title must not be empty");
        }
        if self.entries.is_empty() {
            bail!("pack must contain at least one entry");
        }

        let mut seen = std::collections::HashSet::new();
        for entry in &self.entries {
            if entry.id.trim().is_empty() {
                bail!("entry id must not be empty");
            }
            if !seen.insert(entry.id.as_str()) {
                bail!("duplicate entry id: {}", entry.id);
            }
            if entry.title.trim().is_empty() {
                bail!("entry {} must have a title", entry.id);
            }
            if entry.description.trim().is_empty() {
                bail!("entry {} must have a description", entry.id);
            }
        }

        Ok(())
    }
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn parse_pack_str(input: &str) -> Result<Pack> {
    let pack: Pack = serde_yaml::from_str(input)?;
    pack.validate()?;
    Ok(pack)
}

pub fn parse_pack_file(path: &std::path::Path) -> Result<Pack> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| anyhow!("failed to read {}: {err}", path.display()))?;
    parse_pack_str(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pack_rejects_duplicate_entry_ids() {
        let input = r#"
pack:
  id: "tmux-core"
  tool: "tmux"
  title: "tmux Core"
  version: "0.1.0"
  source: "test"
entries:
  - id: "dup"
    type: "binding"
    title: "First"
    description: "first"
  - id: "dup"
    type: "binding"
    title: "Second"
    description: "second"
"#;

        let err = parse_pack_str(input).expect_err("pack should fail validation");
        assert!(err.to_string().contains("duplicate entry id"));
    }

    #[test]
    fn parse_pack_accepts_minimal_valid_pack() {
        let input = r#"
pack:
  id: "tmux-core"
  tool: "tmux"
  title: "tmux Core"
  version: "0.1.0"
  source: "test"
entries:
  - id: "copy-mode"
    type: "binding"
    title: "Copy Mode"
    description: "Enter copy mode"
"#;

        let pack = parse_pack_str(input).expect("pack should parse");
        assert_eq!(pack.pack.tool, "tmux");
        assert_eq!(pack.entries.len(), 1);
    }
}
