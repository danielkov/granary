//! Watch loop utility for commands with watch mode.
//!
//! Provides a reusable loop that handles terminal clearing, polling,
//! and graceful shutdown via Ctrl+C.

use crate::error::Result;
use std::future::Future;
use std::io::{self, Write};
use std::time::Duration;

/// Runs a function repeatedly with terminal clearing until Ctrl+C.
///
/// # Arguments
/// * `interval` - Duration between polls
/// * `render` - Async function that returns the output string to display
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use crate::error::Result;
///
/// async fn render_status() -> Result<String> {
///     Ok("Current status: OK".to_string())
/// }
///
/// // Run watch loop with 2 second interval
/// watch_loop(Duration::from_secs(2), render_status).await?;
/// ```
pub async fn watch_loop<F, Fut>(interval: Duration, render: F) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<String>>,
{
    loop {
        // Clear terminal (use ANSI escape codes for cross-platform)
        print!("\x1B[2J\x1B[1;1H");

        // Render current state
        match render().await {
            Ok(output) => print!("{}", output),
            Err(e) => eprintln!("Error: {}", e),
        }

        // Flush stdout to ensure output is displayed immediately
        let _ = io::stdout().flush();

        // Wait for interval or Ctrl+C
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\nWatch stopped.");
                break;
            }
            _ = tokio::time::sleep(interval) => {
                // Continue to next iteration
            }
        }
    }
    Ok(())
}

/// Returns a status line showing the watch polling interval.
///
/// # Arguments
/// * `interval` - The polling interval duration
///
/// # Returns
/// A formatted string like "Watching (every 2s) - Press Ctrl+C to stop"
pub fn watch_status_line(interval: Duration) -> String {
    format!(
        "Watching (every {}s) - Press Ctrl+C to stop",
        interval.as_secs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_status_line() {
        let line = watch_status_line(Duration::from_secs(2));
        assert_eq!(line, "Watching (every 2s) - Press Ctrl+C to stop");

        let line = watch_status_line(Duration::from_secs(5));
        assert_eq!(line, "Watching (every 5s) - Press Ctrl+C to stop");
    }
}
