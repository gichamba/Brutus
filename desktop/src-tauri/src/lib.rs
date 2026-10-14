//! Expose recovery commands to the local Tauri window.

/// Numeric ranges, job allocation and cancellable candidate testing.
mod engine;
/// Settings and history records exchanged with the frontend.
mod model;
/// PDF password authentication and protection checks.
mod pdf;
/// Run ownership, file discovery, workers and checkpoints.
mod service;
/// SQLite run history and successful content-hash results.
mod store;

use model::{Run, Settings};
use service::Service;
use std::sync::Arc;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
/// Return selected local paths; cancelling the native picker returns an empty list.
async fn pick_input(app: tauri::AppHandle, folder: bool) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let picker = app.dialog().file();
        let paths = if folder {
            picker.blocking_pick_folder().map(|p| vec![p])
        } else {
            picker
                .add_filter("PDF documents", &["pdf"])
                .blocking_pick_files()
        };
        paths
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                p.into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
/// Validate settings and start a run, rejecting another active run.
fn start_run(
    service: State<'_, Arc<Service>>,
    inputs: Vec<String>,
    settings: Settings,
) -> Result<String, String> {
    service.inner().start(inputs, settings)
}

#[tauri::command]
/// Resume a saved cancelled or interrupted run using its original settings.
fn resume_run(service: State<'_, Arc<Service>>, id: String) -> Result<String, String> {
    service.inner().resume(&id)
}

#[tauri::command]
/// Request cancellation while retaining completed-range checkpoints.
fn cancel_run(service: State<'_, Arc<Service>>) -> Result<(), String> {
    service.cancel()
}

#[tauri::command]
/// Return the current in-memory run with its active elapsed time.
fn get_current(service: State<'_, Arc<Service>>) -> Result<Option<Run>, String> {
    service.snapshot()
}

#[tauri::command]
/// Return saved runs in descending start-time order.
fn get_history(service: State<'_, Arc<Service>>) -> Result<Vec<Run>, String> {
    service.history()
}

#[tauri::command]
/// Suggest a worker count that reserves one available CPU where possible.
fn get_defaults() -> Settings {
    Settings {
        start: 0,
        end: 999999,
        workers: std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).clamp(1, 32))
            .unwrap_or(1),
    }
}

/// Start the desktop shell and initialize its local recovery service.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = service::history_path(app.path().app_data_dir()?)?;
            app.manage(Arc::new(Service::new(&path)?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pick_input,
            start_run,
            resume_run,
            cancel_run,
            get_current,
            get_history,
            get_defaults
        ])
        .run(tauri::generate_context!())
        .expect("Brutus could not initialize its desktop runtime");
}
