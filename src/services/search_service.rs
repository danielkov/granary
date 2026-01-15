use sqlx::SqlitePool;

use crate::db;
use crate::error::Result;
use crate::models::*;

/// Search initiatives, projects, and tasks by query string
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    // Search initiatives first (highest hierarchy level)
    let initiatives = db::search::search_initiatives(pool, query).await?;
    for initiative in initiatives {
        results.push(SearchResult::Initiative {
            id: initiative.id,
            name: initiative.name,
            description: initiative.description,
            status: initiative.status,
        });
    }

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
