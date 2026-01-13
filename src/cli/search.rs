use crate::error::Result;
use crate::output::{Formatter, OutputFormat};
use crate::services::{self, Workspace};

/// Handle search command
pub async fn search(query: &str, format: OutputFormat) -> Result<()> {
    let workspace = Workspace::find()?;
    let pool = workspace.pool().await?;

    let results = services::search(&pool, query).await?;
    let formatter = Formatter::new(format);
    println!("{}", formatter.format_search_results(&results));

    Ok(())
}
