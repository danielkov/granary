//! Runner process management.
//!
//! This module handles spawning and managing runner processes. Each runner
//! is a child process that executes a command in response to an event.
//! Runners capture stdout/stderr to log files and report exit status.

use std::path::Path;
use std::process::Stdio;

use tokio::process::{Child, Command};

use crate::error::{GranaryError, Result};
use crate::models::run::Run;

/// Handle to a spawned runner process.
///
/// This struct tracks a running process and its associated metadata.
/// The caller is responsible for calling `wait()` or `wait_with_timeout()`
/// to collect the process exit status.
pub struct RunnerHandle {
    /// The run ID associated with this process
    pub run_id: String,
    /// The child process handle
    child: Child,
    /// Process ID (captured at spawn time)
    pub pid: u32,
}

impl RunnerHandle {
    /// Get the process ID.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Check if the process has exited without blocking.
    ///
    /// Returns `Some((exit_code, error_message))` if the process has exited,
    /// or `None` if it's still running.
    pub fn try_wait(&mut self) -> Result<Option<(i32, Option<String>)>> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                let exit_code = status.code().unwrap_or(-1);
                let error = if !status.success() {
                    Some(format!("Process exited with code {}", exit_code))
                } else {
                    None
                };
                Ok(Some((exit_code, error)))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(GranaryError::Io(e)),
        }
    }

    /// Wait for the process to exit.
    ///
    /// Returns `(exit_code, error_message)` where error_message is Some
    /// if the process exited with a non-zero code.
    pub async fn wait(mut self) -> Result<(i32, Option<String>)> {
        let status = self.child.wait().await?;
        let exit_code = status.code().unwrap_or(-1);
        let error = if !status.success() {
            Some(format!("Process exited with code {}", exit_code))
        } else {
            None
        };
        Ok((exit_code, error))
    }

    /// Kill the process.
    ///
    /// This sends SIGKILL on Unix or terminates the process on Windows.
    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await.map_err(GranaryError::Io)
    }

    /// Start the process termination (sends SIGKILL).
    ///
    /// This begins killing the process but doesn't wait for it to complete.
    pub fn start_kill(&mut self) -> Result<()> {
        self.child.start_kill().map_err(GranaryError::Io)
    }
}

/// Spawn a runner process for a run.
///
/// # Arguments
/// * `run` - The run record containing command and arguments
/// * `log_dir` - Directory to write log files to
///
/// # Returns
/// A `RunnerHandle` that can be used to track and wait for the process.
///
/// # Log Files
/// The process stdout and stderr are combined and written to a log file
/// at `{log_dir}/{run_id}.log`.
pub async fn spawn_runner(run: &Run, log_dir: &Path) -> Result<RunnerHandle> {
    // Ensure log directory exists
    std::fs::create_dir_all(log_dir)?;

    let log_path = log_dir.join(format!("{}.log", run.id));
    let log_file = std::fs::File::create(&log_path)?;
    let log_file_stderr = log_file.try_clone()?;

    let args = run.args_vec();

    let child = Command::new(&run.command)
        .args(&args)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_stderr))
        .spawn()
        .map_err(|e| {
            GranaryError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to spawn runner '{}': {}", run.command, e),
            ))
        })?;

    let pid = child.id().ok_or_else(|| {
        GranaryError::Conflict("Failed to get PID of spawned process".to_string())
    })?;

    Ok(RunnerHandle {
        run_id: run.id.clone(),
        child,
        pid,
    })
}

/// Spawn a runner process with environment variables.
///
/// # Arguments
/// * `run` - The run record containing command and arguments
/// * `log_dir` - Directory to write log files to
/// * `env_vars` - Environment variables to set for the process
///
/// # Returns
/// A `RunnerHandle` that can be used to track and wait for the process.
pub async fn spawn_runner_with_env(
    run: &Run,
    log_dir: &Path,
    env_vars: &[(String, String)],
) -> Result<RunnerHandle> {
    // Ensure log directory exists
    std::fs::create_dir_all(log_dir)?;

    let log_path = log_dir.join(format!("{}.log", run.id));
    let log_file = std::fs::File::create(&log_path)?;
    let log_file_stderr = log_file.try_clone()?;

    let args = run.args_vec();

    let mut cmd = Command::new(&run.command);
    cmd.args(&args)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_stderr));

    // Add environment variables
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let child = cmd.spawn().map_err(|e| {
        GranaryError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to spawn runner '{}': {}", run.command, e),
        ))
    })?;

    let pid = child.id().ok_or_else(|| {
        GranaryError::Conflict("Failed to get PID of spawned process".to_string())
    })?;

    Ok(RunnerHandle {
        run_id: run.id.clone(),
        child,
        pid,
    })
}

/// Read the contents of a run's log file.
///
/// # Arguments
/// * `run_id` - The run ID
/// * `log_dir` - Directory containing log files
///
/// # Returns
/// The log file contents as a string, or an error if the file doesn't exist.
pub fn read_log(run_id: &str, log_dir: &Path) -> Result<String> {
    let log_path = log_dir.join(format!("{}.log", run_id));
    std::fs::read_to_string(&log_path).map_err(GranaryError::Io)
}

/// Get the path to a run's log file.
///
/// # Arguments
/// * `run_id` - The run ID
/// * `log_dir` - Directory containing log files
///
/// # Returns
/// The path to the log file (may not exist yet).
pub fn log_path(run_id: &str, log_dir: &Path) -> std::path::PathBuf {
    log_dir.join(format!("{}.log", run_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_run(command: &str, args: Vec<&str>) -> Run {
        Run {
            id: "run-test123".to_string(),
            worker_id: "worker-test".to_string(),
            event_id: 1,
            event_type: "task.started".to_string(),
            entity_id: "task-1".to_string(),
            command: command.to_string(),
            args: serde_json::to_string(&args).unwrap(),
            status: "pending".to_string(),
            exit_code: None,
            error_message: None,
            attempt: 1,
            max_attempts: 3,
            next_retry_at: None,
            pid: None,
            log_path: None,
            started_at: None,
            completed_at: None,
            created_at: "2024-01-15T10:00:00Z".to_string(),
            updated_at: "2024-01-15T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_spawn_runner_success() {
        let temp_dir = TempDir::new().unwrap();
        let run = create_test_run("echo", vec!["hello", "world"]);

        let handle = spawn_runner(&run, temp_dir.path()).await.unwrap();
        assert!(!handle.run_id.is_empty());
        assert!(handle.pid > 0);

        let (exit_code, error) = handle.wait().await.unwrap();
        assert_eq!(exit_code, 0);
        assert!(error.is_none());

        // Check log file was created
        let log_content = read_log(&run.id, temp_dir.path()).unwrap();
        assert!(log_content.contains("hello world"));
    }

    #[tokio::test]
    async fn test_spawn_runner_failure() {
        let temp_dir = TempDir::new().unwrap();
        let run = create_test_run("false", vec![]); // 'false' command always exits with 1

        let handle = spawn_runner(&run, temp_dir.path()).await.unwrap();
        let (exit_code, error) = handle.wait().await.unwrap();

        assert_eq!(exit_code, 1);
        assert!(error.is_some());
        assert!(error.unwrap().contains("exited with code 1"));
    }

    #[tokio::test]
    async fn test_spawn_runner_invalid_command() {
        let temp_dir = TempDir::new().unwrap();
        let run = create_test_run("nonexistent_command_12345", vec![]);

        let result = spawn_runner(&run, temp_dir.path()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_wait_running() {
        let temp_dir = TempDir::new().unwrap();
        // Use 'sleep' to have a long-running process
        let run = create_test_run("sleep", vec!["10"]);

        let mut handle = spawn_runner(&run, temp_dir.path()).await.unwrap();

        // Process should still be running
        let result = handle.try_wait().unwrap();
        assert!(result.is_none());

        // Kill the process
        handle.kill().await.unwrap();

        // Now it should be done
        let result = handle.try_wait().unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_log_path() {
        let dir = Path::new("/var/logs");
        let path = log_path("run-abc123", dir);
        assert_eq!(path, Path::new("/var/logs/run-abc123.log"));
    }
}
