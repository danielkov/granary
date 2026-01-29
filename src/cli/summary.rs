use std::time::Duration;

use crate::cli::watch::watch_loop;
use crate::error::Result;
use crate::output::{OutputFormat, json, prompt};
use crate::services::{self, Workspace};

/// Generate summary
pub async fn summary(
    token_budget: Option<usize>,
    format: OutputFormat,
    watch: bool,
    interval: u64,
) -> Result<()> {
    if watch {
        let interval_duration = Duration::from_secs(interval);
        watch_loop(interval_duration, || async {
            render_summary(token_budget, format).await
        })
        .await?;
    } else {
        let output = render_summary(token_budget, format).await?;
        print!("{}", output);
    }

    Ok(())
}

/// Render summary output as a string (for both regular and watch mode)
async fn render_summary(token_budget: Option<usize>, format: OutputFormat) -> Result<String> {
    let workspace = Workspace::find()?;
    let pool = workspace.pool().await?;

    let summary = services::generate_summary(&pool, &workspace, token_budget).await?;

    let output = match format {
        OutputFormat::Json => json::format_summary(&summary),
        OutputFormat::Prompt => prompt::format_summary(&summary),
        _ => format_summary_table(&summary),
    };

    Ok(output)
}

/// Format summary as a table string
fn format_summary_table(summary: &json::SummaryOutput) -> String {
    let mut output = String::new();
    output.push_str("=== Summary ===\n\n");

    if let Some(session) = &summary.session {
        output.push_str(&format!(
            "Session: {} ({})\n",
            session.name.as_deref().unwrap_or("-"),
            session.id
        ));
        if let Some(mode) = &session.mode {
            output.push_str(&format!("Mode: {}\n", mode));
        }
        if let Some(focus) = &session.focus_task_id {
            output.push_str(&format!("Focus: {}\n", focus));
        }
        output.push('\n');
    }

    output.push_str("State of Work:\n");
    output.push_str(&format!("  Total tasks: {}\n", summary.state.total_tasks));
    output.push_str(&format!(
        "  By status: {} todo, {} in progress, {} done, {} blocked\n",
        summary.state.by_status.todo,
        summary.state.by_status.in_progress,
        summary.state.by_status.done,
        summary.state.by_status.blocked
    ));
    output.push_str(&format!(
        "  By priority: {} P0, {} P1, {} P2, {} P3, {} P4\n",
        summary.state.by_priority.p0,
        summary.state.by_priority.p1,
        summary.state.by_priority.p2,
        summary.state.by_priority.p3,
        summary.state.by_priority.p4
    ));
    output.push('\n');

    if let Some(focus) = &summary.focus_task {
        output.push_str("Focus Task:\n");
        output.push_str(&format!("  {} ({})\n", focus.title, focus.id));
        output.push_str(&format!(
            "  Status: {} | Priority: {}\n",
            focus.status, focus.priority
        ));
        if let Some(desc) = &focus.description {
            output.push_str(&format!("  {}\n", desc));
        }
        output.push('\n');
    }

    if !summary.blockers.is_empty() {
        output.push_str(&format!("Blockers ({}):\n", summary.blockers.len()));
        for task in &summary.blockers {
            output.push_str(&format!("  - {} ({})", task.title, task.id));
            if let Some(reason) = &task.blocked_reason {
                output.push_str(&format!(": {}", reason));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    if !summary.next_actions.is_empty() {
        output.push_str("Next Actions:\n");
        for task in &summary.next_actions {
            output.push_str(&format!(
                "  - [{}] {} ({})\n",
                task.priority, task.title, task.id
            ));
        }
        output.push('\n');
    }

    if !summary.recent_decisions.is_empty() {
        output.push_str("Recent Decisions:\n");
        for comment in &summary.recent_decisions {
            let author = comment.author.as_deref().unwrap_or("unknown");
            output.push_str(&format!("  - {}: {}\n", author, comment.content));
        }
        output.push('\n');
    }

    output
}

/// Generate context pack
pub async fn context(
    include: Option<String>,
    max_items: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    let workspace = Workspace::find()?;
    let pool = workspace.pool().await?;

    let include_vec = include.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    let context = services::generate_context(&pool, &workspace, include_vec, max_items).await?;

    match format {
        OutputFormat::Json => {
            println!("{}", json::format_context(&context));
        }
        OutputFormat::Prompt => {
            println!("{}", prompt::format_context(&context));
        }
        _ => {
            // Default to prompt format for context
            println!("{}", prompt::format_context(&context));
        }
    }

    Ok(())
}

/// Generate handoff document
pub async fn handoff(
    to: &str,
    tasks: &str,
    constraints: Option<String>,
    acceptance_criteria: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let workspace = Workspace::find()?;
    let pool = workspace.pool().await?;

    let task_ids: Vec<String> = tasks.split(',').map(|s| s.trim().to_string()).collect();

    let handoff = services::generate_handoff(
        &pool,
        to,
        &task_ids,
        constraints.as_deref(),
        acceptance_criteria.as_deref(),
        None,
    )
    .await?;

    match format {
        OutputFormat::Json => {
            println!("{}", json::format_handoff(&handoff));
        }
        OutputFormat::Prompt => {
            println!("{}", prompt::format_handoff(&handoff));
        }
        _ => {
            println!("{}", prompt::format_handoff(&handoff));
        }
    }

    Ok(())
}
