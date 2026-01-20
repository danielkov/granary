//! Granary daemon - manages worker lifecycles via IPC.
//!
//! The granaryd binary is a long-running background process that:
//! - Accepts IPC connections from the CLI over Unix domain sockets
//! - Manages worker lifecycles (start, stop, query)
//! - Handles graceful shutdown on SIGTERM/SIGINT
//! - Restores workers that were running before the daemon stopped
//!
//! ## Usage
//!
//! The daemon is typically started automatically by the CLI when needed.
//! Manual start: `granaryd`
//!
//! ## Files
//!
//! - `~/.granary/daemon/granaryd.sock` - Unix socket for IPC
//! - `~/.granary/daemon/granaryd.pid` - PID file for process tracking
//! - `~/.granary/daemon/daemon.log` - Daemon log file

use std::sync::Arc;

use tokio::select;
use tokio::signal::unix::{SignalKind, signal};

use granary::daemon::IpcConnection;
use granary::daemon::listener::IpcListener;
use granary::daemon::protocol::{Operation, Request, Response};
use granary::daemon::worker_manager::WorkerManager;
use granary::services::global_config as global_config_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ensure daemon directory exists
    let daemon_dir = global_config_service::daemon_dir()?;
    std::fs::create_dir_all(&daemon_dir)?;

    // Initialize logging to daemon log file
    let _log_path = global_config_service::daemon_log_path()?;
    // TODO: Set up file-based logging (tracing-subscriber with file appender)

    // Write PID file
    let pid_path = global_config_service::daemon_pid_path()?;
    std::fs::write(&pid_path, std::process::id().to_string())?;

    // Open global database
    let global_pool = global_config_service::global_pool().await?;

    // Create worker manager
    let manager = Arc::new(WorkerManager::new(global_pool));

    // Restore workers that were running before daemon stopped
    if let Err(e) = manager.restore_workers().await {
        eprintln!("Warning: Failed to restore workers: {}", e);
    }

    // Start IPC listener
    let socket_path = global_config_service::daemon_socket_path()?;
    let listener = IpcListener::bind(&socket_path).await?;

    eprintln!("granaryd listening on {:?}", listener.socket_path());

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    // Flag to track shutdown request from IPC
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Main loop
    loop {
        // Check if shutdown was requested via IPC
        if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
            eprintln!("Shutdown requested via IPC");
            break;
        }

        select! {
            // Handle shutdown signals
            _ = sigterm.recv() => {
                eprintln!("Received SIGTERM, shutting down...");
                break;
            }
            _ = sigint.recv() => {
                eprintln!("Received SIGINT, shutting down...");
                break;
            }

            // Accept new connections
            result = listener.accept() => {
                match result {
                    Ok(conn) => {
                        let manager = Arc::clone(&manager);
                        let shutdown_flag = Arc::clone(&shutdown_flag);
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(conn, &manager, &shutdown_flag).await {
                                eprintln!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
        }
    }

    // Graceful shutdown
    eprintln!("Shutting down workers...");
    manager.shutdown_all().await?;

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_path);

    eprintln!("granaryd shutdown complete");
    Ok(())
}

/// Handle a single client connection.
///
/// Processes requests in a loop until the connection is closed or
/// a shutdown operation is received.
async fn handle_connection(
    mut conn: IpcConnection,
    manager: &WorkerManager,
    shutdown_flag: &std::sync::atomic::AtomicBool,
) -> anyhow::Result<()> {
    loop {
        let request = match conn.recv_request().await {
            Ok(req) => req,
            Err(_) => break, // Connection closed
        };

        let (response, should_shutdown) = dispatch_request(request, manager).await;
        conn.send_response(&response).await?;

        if should_shutdown {
            // Signal the main loop to shutdown
            shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            break;
        }
    }
    Ok(())
}

/// Dispatch a request to the appropriate handler.
///
/// Returns the response and a flag indicating if the daemon should shutdown.
async fn dispatch_request(request: Request, manager: &WorkerManager) -> (Response, bool) {
    let id = request.id;

    match request.op {
        Operation::Ping => {
            let response = Response::ok(
                id,
                serde_json::json!({
                    "version": env!("CARGO_PKG_VERSION"),
                    "status": "running"
                }),
            );
            (response, false)
        }

        Operation::Shutdown => {
            // Return acknowledgment, then daemon will shutdown
            let response = Response::ok(id, "shutdown_ack");
            (response, true)
        }

        Operation::StartWorker(req) => {
            let create = granary::models::worker::CreateWorker {
                runner_name: req.runner_name,
                command: req.command,
                args: req.args,
                event_type: req.event_type,
                filters: req.filters,
                concurrency: req.concurrency,
                instance_path: req.instance_path,
                detached: !req.attach,
            };

            match manager.start_worker(create).await {
                Ok(worker) => {
                    let response = Response::ok(id, &worker);
                    (response, false)
                }
                Err(e) => {
                    let response = Response::err(id, e.to_string());
                    (response, false)
                }
            }
        }

        Operation::StopWorker {
            worker_id,
            stop_runs,
        } => match manager.stop_worker(&worker_id, stop_runs).await {
            Ok(_) => (Response::ok_empty(id), false),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::GetWorker { worker_id } => match manager.get_worker(&worker_id).await {
            Ok(Some(worker)) => (Response::ok(id, &worker), false),
            Ok(None) => (
                Response::err(id, format!("Worker {} not found", worker_id)),
                false,
            ),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::ListWorkers { all } => match manager.list_workers(all).await {
            Ok(workers) => (Response::ok(id, &workers), false),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::PruneWorkers => {
            // TODO: Implement pruning of stopped/errored workers
            (Response::err(id, "Operation not yet implemented"), false)
        }

        Operation::WorkerLogs {
            worker_id,
            follow: _,
            lines,
        } => {
            // Get worker log path and read logs
            match manager.get_worker_log_path(&worker_id) {
                Ok(path) => {
                    if path.exists() {
                        match read_log_tail(&path, lines as usize) {
                            Ok(logs) => {
                                (Response::ok(id, serde_json::json!({ "logs": logs })), false)
                            }
                            Err(e) => (
                                Response::err(id, format!("Failed to read logs: {}", e)),
                                false,
                            ),
                        }
                    } else {
                        (
                            Response::ok(
                                id,
                                serde_json::json!({ "logs": "", "message": "No log file found" }),
                            ),
                            false,
                        )
                    }
                }
                Err(e) => (Response::err(id, e.to_string()), false),
            }
        }

        Operation::GetRun { run_id } => match manager.get_run(&run_id).await {
            Ok(Some(run)) => (Response::ok(id, &run), false),
            Ok(None) => (
                Response::err(id, format!("Run {} not found", run_id)),
                false,
            ),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::ListRuns {
            worker_id,
            status,
            all,
        } => {
            match manager
                .list_runs(worker_id.as_deref(), status.as_deref(), all)
                .await
            {
                Ok(runs) => (Response::ok(id, &runs), false),
                Err(e) => (Response::err(id, e.to_string()), false),
            }
        }

        Operation::StopRun { run_id } => match manager.stop_run(&run_id).await {
            Ok(()) => (Response::ok_empty(id), false),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::PauseRun { run_id } => match manager.pause_run(&run_id).await {
            Ok(()) => (Response::ok_empty(id), false),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::ResumeRun { run_id } => match manager.resume_run(&run_id).await {
            Ok(()) => (Response::ok_empty(id), false),
            Err(e) => (Response::err(id, e.to_string()), false),
        },

        Operation::RunLogs {
            run_id,
            follow: _,
            lines,
        } => {
            // Get log path and read logs
            match manager.get_run_log_path(&run_id).await {
                Ok(Some(path)) => {
                    // Read last N lines from log file
                    match read_log_tail(&path, lines as usize) {
                        Ok(logs) => (Response::ok(id, serde_json::json!({ "logs": logs })), false),
                        Err(e) => (
                            Response::err(id, format!("Failed to read logs: {}", e)),
                            false,
                        ),
                    }
                }
                Ok(None) => (
                    Response::ok(
                        id,
                        serde_json::json!({ "logs": "", "message": "No log file found" }),
                    ),
                    false,
                ),
                Err(e) => (Response::err(id, e.to_string()), false),
            }
        }

        Operation::GetLogs(req) => {
            match manager
                .get_logs(&req.target_id, req.target_type, req.since_line, req.limit)
                .await
            {
                Ok(response) => (Response::ok(id, &response), false),
                Err(e) => (Response::err(id, e.to_string()), false),
            }
        }
    }
}

/// Read the last N lines from a log file
fn read_log_tail(path: &std::path::Path, lines: usize) -> std::io::Result<String> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().collect::<std::io::Result<_>>()?;

    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };

    Ok(all_lines[start..].join("\n"))
}
