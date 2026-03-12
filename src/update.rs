use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::Command as ProcessCommand,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

const RELEASE_API_URL: &str = "https://api.github.com/repos/younesehb/bindfinder/releases/latest";
const INSTALLER_URL: &str =
    "https://github.com/younesehb/bindfinder/releases/latest/download/install.sh";
const CACHE_TTL_SECONDS: u64 = 60 * 60 * 12;
const USER_AGENT: &str = "bindfinder";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UpdateCache {
    checked_at: u64,
    latest_version: String,
    release_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

pub fn cached_or_fetch(current_version: &str) -> Option<UpdateInfo> {
    let cache = load_cache();
    let now = unix_now().ok()?;

    if let Some(cache) = cache.as_ref() {
        if now.saturating_sub(cache.checked_at) <= CACHE_TTL_SECONDS {
            return build_update_info(current_version, &cache.latest_version, &cache.release_url);
        }
    }

    match fetch_latest_release() {
        Ok(cache) => {
            let _ = save_cache(&cache);
            build_update_info(current_version, &cache.latest_version, &cache.release_url)
        }
        Err(_) => cache.as_ref().and_then(|cache| {
            build_update_info(current_version, &cache.latest_version, &cache.release_url)
        }),
    }
}

pub fn check_now(current_version: &str) -> Result<Option<UpdateInfo>> {
    let cache = fetch_latest_release()?;
    save_cache(&cache)?;
    Ok(build_update_info(
        current_version,
        &cache.latest_version,
        &cache.release_url,
    ))
}

pub fn perform_update(current_version: &str) -> Result<Option<UpdateInfo>> {
    let info = check_now(current_version)?;
    let Some(info) = info else {
        return Ok(None);
    };

    let installer = download_installer()?;
    let status = ProcessCommand::new("sh")
        .arg(&installer)
        .arg("--no-setup")
        .status()
        .context("failed to run installer")?;
    let _ = fs::remove_file(&installer);

    if !status.success() {
        return Err(anyhow!("installer exited with {}", status));
    }

    Ok(Some(info))
}

fn fetch_latest_release() -> Result<UpdateCache> {
    let response = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(2))
        .timeout_read(Duration::from_secs(2))
        .timeout_write(Duration::from_secs(2))
        .build()
        .get(RELEASE_API_URL)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", USER_AGENT)
        .call()
        .context("failed to fetch latest release metadata")?;

    let release: GitHubRelease = response
        .into_json()
        .context("failed to parse latest release metadata")?;
    Ok(UpdateCache {
        checked_at: unix_now()?,
        latest_version: normalize_tag(&release.tag_name),
        release_url: release.html_url,
    })
}

fn download_installer() -> Result<PathBuf> {
    let response = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(20))
        .build()
        .get(INSTALLER_URL)
        .set("User-Agent", USER_AGENT)
        .call()
        .context("failed to download installer")?;

    let body = response
        .into_string()
        .context("failed to read installer response")?;
    let path = std::env::temp_dir().join(format!("bindfinder-update-{}.sh", unix_now()?));
    let mut file =
        fs::File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
    file.write_all(body.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

fn build_update_info(
    current_version: &str,
    latest_version: &str,
    release_url: &str,
) -> Option<UpdateInfo> {
    if compare_versions(latest_version, current_version).is_gt() {
        Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            release_url: release_url.to_string(),
        })
    } else {
        None
    }
}

fn load_cache() -> Option<UpdateCache> {
    let path = cache_path()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_cache(cache: &UpdateCache) -> Result<()> {
    let Some(path) = cache_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let content = serde_json::to_string(cache)?;
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn cache_path() -> Option<PathBuf> {
    paths::bindfinder_cache_file("updates.json")
}

fn unix_now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?
        .as_secs())
}

fn normalize_tag(tag: &str) -> String {
    tag.strip_prefix('v').unwrap_or(tag).to_string()
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = parse_version(left);
    let right_parts = parse_version(right);
    left_parts.cmp(&right_parts)
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::compare_versions;

    #[test]
    fn compare_versions_orders_semver_like_values() {
        assert!(compare_versions("0.1.10", "0.1.9").is_gt());
        assert!(compare_versions("1.0.0", "0.9.9").is_gt());
        assert!(compare_versions("0.1.9", "0.1.9").is_eq());
        assert!(compare_versions("0.1.8", "0.1.9").is_lt());
    }
}
