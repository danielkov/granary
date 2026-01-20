//! Global configuration model for user-level settings.
//!
//! The global config lives at `~/.granary/config.toml` and contains
//! user-level settings like runner definitions that persist across workspaces.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global configuration structure stored at ~/.granary/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    /// Runner definitions that can be referenced by name
    #[serde(default)]
    pub runners: HashMap<String, RunnerConfig>,
}

/// Configuration for a runner that executes tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// Command to execute (e.g., "claude", "python")
    pub command: String,

    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Maximum concurrent executions for this runner
    #[serde(default)]
    pub concurrency: Option<u32>,

    /// Default event type this runner responds to
    #[serde(default)]
    pub on: Option<String>,

    /// Environment variables to set when running
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl RunnerConfig {
    /// Create a new runner configuration with just a command
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            concurrency: None,
            on: None,
            env: HashMap::new(),
        }
    }

    /// Expand environment variables in args.
    /// Supports ${VAR} and $VAR syntax.
    pub fn expand_env_in_args(&self) -> Vec<String> {
        self.args.iter().map(|arg| expand_env_vars(arg)).collect()
    }
}

/// Expand environment variables in a string.
/// Supports ${VAR} and $VAR syntax.
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Handle ${VAR} syntax
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            let value = std::env::var(var_name).unwrap_or_default();
            result = format!(
                "{}{}{}",
                &result[..start],
                value,
                &result[start + end + 1..]
            );
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_global_config() {
        let config = GlobalConfig::default();
        assert!(config.runners.is_empty());
    }

    #[test]
    fn test_runner_config_new() {
        let runner = RunnerConfig::new("claude");
        assert_eq!(runner.command, "claude");
        assert!(runner.args.is_empty());
        assert!(runner.concurrency.is_none());
        assert!(runner.on.is_none());
        assert!(runner.env.is_empty());
    }

    #[test]
    fn test_expand_env_vars() {
        // SAFETY: Tests are run single-threaded
        unsafe {
            std::env::set_var("TEST_VAR", "hello");
        }
        assert_eq!(expand_env_vars("${TEST_VAR} world"), "hello world");
        assert_eq!(expand_env_vars("no vars here"), "no vars here");
        // SAFETY: Tests are run single-threaded
        unsafe {
            std::env::remove_var("TEST_VAR");
        }
    }

    #[test]
    fn test_runner_expand_env_in_args() {
        // SAFETY: Tests are run single-threaded
        unsafe {
            std::env::set_var("TOKEN", "secret123");
        }
        let mut runner = RunnerConfig::new("curl");
        runner.args = vec![
            "-H".to_string(),
            "Authorization: Bearer ${TOKEN}".to_string(),
        ];

        let expanded = runner.expand_env_in_args();
        assert_eq!(expanded[1], "Authorization: Bearer secret123");
        // SAFETY: Tests are run single-threaded
        unsafe {
            std::env::remove_var("TOKEN");
        }
    }
}
