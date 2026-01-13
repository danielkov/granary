use sqlx::SqlitePool;

use crate::db;
use crate::error::Result;
use crate::models::*;

/// Search both projects and tasks by query string
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    // Search projects
    let projects = db::search::search_projects(pool, query).await?;
    for project in projects {
        results.push(SearchResult::Project {
            id: project.id,
            name: project.name,
            description: project.description,
            status: project.status,
        });
    }

    // Search tasks
    let tasks = db::search::search_tasks(pool, query).await?;
    for task in tasks {
        results.push(SearchResult::Task {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            project_id: task.project_id,
        });
    }

    Ok(results)
}
