use std::path::PathBuf;
use std::process::Command;

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::error::{GranaryError, Result};

const GITHUB_REPO: &str = "danielkov/granary";
const CACHE_TTL_HOURS: i64 = 24;

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateCache {
    last_check: DateTime<Utc>,
    latest_version: String,
}

/// Get cache file path (~/.granary/update-check.json)
fn cache_path() -> Option<PathBuf> {
    #[cfg(unix)]
    let home = std::env::var("HOME").ok();

    #[cfg(windows)]
    let home = std::env::var("USERPROFILE").ok();

    home.map(|h| PathBuf::from(h).join(".granary").join("update-check.json"))
}

/// Read cached update info (if fresh, <24h old)
fn read_cache() -> Option<UpdateCache> {
    let path = cache_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let cache: UpdateCache = serde_json::from_str(&content).ok()?;

    // Check if cache is still fresh
    let age = Utc::now().signed_duration_since(cache.last_check);
    if age.num_hours() < CACHE_TTL_HOURS {
        Some(cache)
    } else {
        None
    }
}

/// Write update cache
fn write_cache(latest: &str) -> Result<()> {
    let Some(path) = cache_path() else {
        return Ok(()); // Silently skip if no home dir
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let cache = UpdateCache {
        last_check: Utc::now(),
        latest_version: latest.to_string(),
    };

    let content = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Fetch latest version from GitHub API
async fn fetch_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "granary-cli")
        .send()
        .await
        .map_err(|e| GranaryError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(GranaryError::Network(format!(
            "GitHub API returned status {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| GranaryError::Network(e.to_string()))?;

    // Strip leading 'v' if present
    let version = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    Ok(version.to_string())
}

/// Compare two version strings using semver
fn is_newer_version(current: &str, latest: &str) -> bool {
    match (Version::parse(current), Version::parse(latest)) {
        (Ok(c), Ok(l)) => l > c,
        _ => latest > current, // Fallback to string comparison
    }
}

/// Check for update (fetches from GitHub and updates cache)
pub async fn check_for_update() -> Result<Option<String>> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = fetch_latest_version().await?;

    // Update cache
    let _ = write_cache(&latest);

    if is_newer_version(current, &latest) {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

/// Check for update using cache only (for version display)
pub fn check_for_update_cached() -> Option<String> {
    let current = env!("CARGO_PKG_VERSION");
    let cache = read_cache()?;

    if is_newer_version(current, &cache.latest_version) {
        Some(cache.latest_version)
    } else {
        None
    }
}

/// Get version string with update notice for clap
pub fn version_with_update_notice() -> &'static str {
    let version = env!("CARGO_PKG_VERSION");

    if let Some(latest) = check_for_update_cached() {
        let s = format!(
            "{}\nUpdate available: {} (run `granary update` to install)",
            version, latest
        );
        Box::leak(s.into_boxed_str())
    } else {
        version
    }
}

/// Run the install script to perform the update
fn run_install_script() -> Result<()> {
    #[cfg(unix)]
    {
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -sSfL https://raw.githubusercontent.com/danielkov/granary/main/scripts/install.sh | sh")
            .status()
            .map_err(|e| GranaryError::Update(format!("Failed to run install script: {}", e)))?;

        if !status.success() {
            return Err(GranaryError::Update("Install script failed".to_string()));
        }
    }

    #[cfg(windows)]
    {
        let status = Command::new("powershell")
            .arg("-Command")
            .arg("irm https://raw.githubusercontent.com/danielkov/granary/main/scripts/install.ps1 | iex")
            .status()
            .map_err(|e| GranaryError::Update(format!("Failed to run install script: {}", e)))?;

        if !status.success() {
            return Err(GranaryError::Update("Install script failed".to_string()));
        }
    }

    Ok(())
}

/// Main update command handler
pub async fn update(check_only: bool) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    println!("Checking for updates...");

    let latest = match fetch_latest_version().await {
        Ok(v) => v,
        Err(e) => {
            return Err(GranaryError::Update(format!(
                "Failed to check for updates: {}",
                e
            )));
        }
    };

    // Update cache
    let _ = write_cache(&latest);

    if !is_newer_version(current, &latest) {
        println!("granary {} is the latest version", current);
        return Ok(());
    }

    if check_only {
        println!("Current version: {}", current);
        println!("Latest version:  {}", latest);
        println!("\nRun `granary update` to install.");
        return Ok(());
    }

    println!("Updating granary {} → {}...", current, latest);
    println!();

    run_install_script()?;

    println!();
    println!("Successfully updated to granary {}!", latest);

    Ok(())
}
