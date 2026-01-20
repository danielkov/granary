//! Global configuration service for loading and saving user-level settings.
//!
//! Manages the config file at `~/.granary/config.toml` and the global database
//! at `~/.granary/workers.db`.

use crate::db::connection::{create_pool, run_migrations};
use crate::error::{GranaryError, Result};
use crate::models::global_config::{GlobalConfig, RunnerConfig};
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Get the global granary config directory (~/.granary)
pub fn config_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".granary"))
        .ok_or_else(|| GranaryError::GlobalConfig("Could not determine home directory".into()))
}

/// Get the path to the global config file (~/.granary/config.toml)
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Get the path to the global workers database (~/.granary/workers.db)
pub fn global_db_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("workers.db"))
}

/// Get the path to the logs directory (~/.granary/logs)
pub fn logs_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("logs"))
}

/// Get the daemon directory (~/.granary/daemon)
pub fn daemon_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("daemon"))
}

/// Get the daemon socket path (~/.granary/daemon/granaryd.sock)
#[cfg(unix)]
pub fn daemon_socket_path() -> Result<PathBuf> {
    Ok(daemon_dir()?.join("granaryd.sock"))
}

/// Get the daemon pipe name (Windows)
///
/// Returns a named pipe path in the format `\\.\pipe\granaryd-{username}`.
/// The username is included for per-user isolation, ensuring each user
/// has their own daemon instance.
#[cfg(windows)]
pub fn daemon_pipe_name() -> String {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    format!(r"\\.\pipe\granaryd-{}", username)
}

/// Get the daemon PID file path (~/.granary/daemon/granaryd.pid)
pub fn daemon_pid_path() -> Result<PathBuf> {
    Ok(daemon_dir()?.join("granaryd.pid"))
}

/// Get the daemon log path (~/.granary/daemon/daemon.log)
pub fn daemon_log_path() -> Result<PathBuf> {
    Ok(daemon_dir()?.join("daemon.log"))
}

/// Get the logs directory for a specific worker (~/.granary/logs/<worker_id>)
pub fn worker_logs_dir(worker_id: &str) -> Result<PathBuf> {
    Ok(logs_dir()?.join(worker_id))
}

/// Get a connection pool to the global workers database
/// Creates the database and runs migrations if it doesn't exist
pub async fn global_pool() -> Result<SqlitePool> {
    let db_path = global_db_path()?;

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let pool = create_pool(&db_path).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

/// Load the global configuration from ~/.granary/config.toml
/// Returns default config if file doesn't exist.
pub fn load() -> Result<GlobalConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }

    let content = std::fs::read_to_string(&path)?;
    toml::from_str(&content)
        .map_err(|e| GranaryError::GlobalConfig(format!("Failed to parse config: {}", e)))
}

/// Save the global configuration to ~/.granary/config.toml
pub fn save(config: &GlobalConfig) -> Result<()> {
    let path = config_path()?;

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|e| GranaryError::GlobalConfig(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(&path, content)?;
    Ok(())
}

/// Get a specific runner by name
pub fn get_runner(name: &str) -> Result<Option<RunnerConfig>> {
    let config = load()?;
    Ok(config.runners.get(name).cloned())
}

/// Add or update a runner configuration
pub fn set_runner(name: &str, runner: RunnerConfig) -> Result<()> {
    let mut config = load()?;
    config.runners.insert(name.to_string(), runner);
    save(&config)
}

/// Remove a runner configuration
pub fn remove_runner(name: &str) -> Result<bool> {
    let mut config = load()?;
    let removed = config.runners.remove(name).is_some();
    if removed {
        save(&config)?;
    }
    Ok(removed)
}

/// List all runner names
pub fn list_runners() -> Result<Vec<String>> {
    let config = load()?;
    Ok(config.runners.keys().cloned().collect())
}

/// Open the config file in the user's editor
pub fn edit_config() -> Result<()> {
    let path = config_path()?;

    // Ensure the directory and file exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create default config file if it doesn't exist
    if !path.exists() {
        let default_config = GlobalConfig::default();
        save(&default_config)?;
    }

    // Get the editor from environment
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    // Open the editor
    let status = std::process::Command::new(&editor).arg(&path).status()?;

    if !status.success() {
        return Err(GranaryError::GlobalConfig(format!(
            "Editor '{}' exited with non-zero status",
            editor
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        assert!(dir.ends_with(".granary"));
    }

    #[test]
    fn test_config_path() {
        let path = config_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("config.toml"));
    }

    #[test]
    fn test_daemon_dir() {
        let dir = daemon_dir();
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        assert!(dir.ends_with("daemon"));
        assert!(dir.parent().unwrap().ends_with(".granary"));
    }

    #[cfg(unix)]
    #[test]
    fn test_daemon_socket_path() {
        let path = daemon_socket_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("granaryd.sock"));
    }

    #[cfg(windows)]
    #[test]
    fn test_daemon_pipe_name() {
        let pipe_name = daemon_pipe_name();
        assert!(pipe_name.starts_with(r"\\.\pipe\granaryd-"));
        // Verify it contains a username component
        assert!(pipe_name.len() > r"\\.\pipe\granaryd-".len());
    }

    #[test]
    fn test_daemon_pid_path() {
        let path = daemon_pid_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("granaryd.pid"));
    }

    #[test]
    fn test_daemon_log_path() {
        let path = daemon_log_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("daemon.log"));
    }
}
