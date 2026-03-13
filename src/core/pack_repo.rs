use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use crate::core::{pack::parse_pack_file, pack::Pack};

pub fn load_repo(root: &Path) -> Result<Vec<Pack>> {
    discover_pack_files(root)?
        .into_iter()
        .map(|path| parse_pack_file(&path))
        .collect()
}

pub fn discover_pack_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_pack_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_pack_files(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
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
            walk_pack_files(&path, acc)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "yml" | "yaml"))
        {
            acc.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_yaml_recursively() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bindfinder-pack-repo-{stamp}"));
        let nested = dir.join("nested");
        fs::create_dir_all(&nested).expect("nested dir");
        fs::write(dir.join("ignore.txt"), "x").expect("ignore write");
        fs::write(nested.join("tmux.yaml"), "pack:\n  id: x\n  tool: x\n  title: x\n  version: x\n  source: x\nentries:\n  - id: a\n    type: binding\n    title: a\n    description: a\n").expect("yaml write");

        let files = discover_pack_files(&dir).expect("discover");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("tmux.yaml"));

        let _ = fs::remove_dir_all(&dir);
    }
}
