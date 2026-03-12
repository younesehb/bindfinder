use std::{env, path::PathBuf};

pub fn config_root() -> Option<PathBuf> {
    if let Some(path) = explicit_dir("XDG_CONFIG_HOME") {
        return Some(path);
    }
    dirs::config_dir()
}

pub fn cache_root() -> Option<PathBuf> {
    if let Some(path) = explicit_dir("XDG_CACHE_HOME") {
        return Some(path);
    }
    dirs::cache_dir()
}

pub fn data_root() -> Option<PathBuf> {
    if let Some(path) = explicit_dir("XDG_DATA_HOME") {
        return Some(path);
    }
    dirs::data_local_dir().or_else(dirs::data_dir)
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn local_share_root() -> Option<PathBuf> {
    if let Some(path) = explicit_dir("XDG_DATA_HOME") {
        return Some(path);
    }
    home_dir().map(|home| home.join(".local").join("share"))
}

pub fn bindfinder_config_file(name: &str) -> Option<PathBuf> {
    config_root().map(|root| root.join("bindfinder").join(name))
}

pub fn bindfinder_config_dir(name: &str) -> Option<PathBuf> {
    config_root().map(|root| root.join("bindfinder").join(name))
}

pub fn bindfinder_data_dir(name: &str) -> Option<PathBuf> {
    data_root().map(|root| root.join("bindfinder").join(name))
}

pub fn bindfinder_cache_file(name: &str) -> Option<PathBuf> {
    cache_root().map(|root| root.join("bindfinder").join(name))
}

fn explicit_dir(var: &str) -> Option<PathBuf> {
    env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
