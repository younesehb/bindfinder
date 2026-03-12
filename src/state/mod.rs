use std::{
    collections::HashSet,
    env, fs,
    path::PathBuf,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserState {
    #[serde(default)]
    pub favorite_entries: HashSet<String>,
    #[serde(default)]
    pub hidden_entries: HashSet<String>,
    #[serde(default)]
    pub favorite_tools: HashSet<String>,
    #[serde(default)]
    pub hidden_tools: HashSet<String>,
}

impl UserState {
    pub fn load() -> Result<Self> {
        let Some(path) = default_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let state = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = default_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let content = serde_yaml::to_string(self)?;
        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    pub fn is_entry_favorite(&self, qualified_id: &str) -> bool {
        self.favorite_entries.contains(qualified_id)
    }

    pub fn is_entry_hidden(&self, qualified_id: &str) -> bool {
        self.hidden_entries.contains(qualified_id)
    }

    pub fn is_tool_favorite(&self, tool: &str) -> bool {
        self.favorite_tools.contains(tool)
    }

    pub fn is_tool_hidden(&self, tool: &str) -> bool {
        self.hidden_tools.contains(tool)
    }

    pub fn toggle_entry_favorite(&mut self, qualified_id: &str) -> bool {
        if self.favorite_entries.remove(qualified_id) {
            false
        } else {
            self.hidden_entries.remove(qualified_id);
            self.favorite_entries.insert(qualified_id.to_string());
            true
        }
    }

    pub fn toggle_entry_hidden(&mut self, qualified_id: &str) -> bool {
        if self.hidden_entries.remove(qualified_id) {
            false
        } else {
            self.favorite_entries.remove(qualified_id);
            self.hidden_entries.insert(qualified_id.to_string());
            true
        }
    }

    pub fn toggle_tool_favorite(&mut self, tool: &str) -> bool {
        if self.favorite_tools.remove(tool) {
            false
        } else {
            self.hidden_tools.remove(tool);
            self.favorite_tools.insert(tool.to_string());
            true
        }
    }

    pub fn toggle_tool_hidden(&mut self, tool: &str) -> bool {
        if self.hidden_tools.remove(tool) {
            false
        } else {
            self.favorite_tools.remove(tool);
            self.hidden_tools.insert(tool.to_string());
            true
        }
    }
}

fn default_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("BINDFINDER_STATE") {
        let path = PathBuf::from(path);
        if !path.as_os_str().is_empty() {
            return Some(path);
        }
    }

    paths::bindfinder_config_file("state.yaml")
}
