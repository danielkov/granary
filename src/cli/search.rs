use std::time::Duration;

use crate::cli::watch::{watch_loop, watch_status_line};
use crate::error::Result;
use crate::output::{Formatter, OutputFormat};
use crate::services::{self, Workspace};

/// Handle search command
pub async fn search(query: &str, format: OutputFormat, watch: bool, interval: u64) -> Result<()> {
    if watch {
        let interval_duration = Duration::from_secs(interval);
        let query = query.to_string();

        watch_loop(interval_duration, || async {
            let output = fetch_and_format_search(&query, format).await?;
            Ok(format!(
                "{}\n\n{}",
                watch_status_line(interval_duration),
                output
            ))
        })
        .await?;
    } else {
        let output = fetch_and_format_search(query, format).await?;
        println!("{}", output);
    }

    Ok(())
}

/// Fetch search results and format them for display
async fn fetch_and_format_search(query: &str, format: OutputFormat) -> Result<String> {
    let workspace = Workspace::find()?;
    let pool = workspace.pool().await?;

    let results = services::search(&pool, query).await?;
    let formatter = Formatter::new(format);
    Ok(formatter.format_search_results(&results))
}
