use std::sync::Arc;

use tauri::State;
use workflow_core::pb::{LoadDataConfig, LoadDataResult};
use workflow_core::{LocalDockerExecutor, WorkflowExecutor};

/// Application state: holds the [`WorkflowExecutor`] currently in use.
///
/// Currently fixed to [`LocalDockerExecutor`]; if switching to K8s in the
/// future, only this needs to be replaced with
/// `Arc::new(K8sJobExecutor::new(...))` — the [`handle_load_data`] command
/// below requires no changes.
struct AppState {
    executor: Arc<dyn WorkflowExecutor>,
}

/// Image name for the LoadData node.
const LOAD_DATA_IMAGE: &str = "proteomic/load-data:latest";

/// Trigger the LoadData node: run the LoadData algorithm in a local Docker
/// container and return the parsed [`LoadDataResult`] (spectrum count,
/// status, output data reference, etc.).
#[tauri::command]
async fn handle_load_data(
    state: State<'_, AppState>,
    config: LoadDataConfig,
) -> Result<LoadDataResult, String> {
    state
        .executor
        .run_load_data(&config)
        .await
        .map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            executor: Arc::new(LocalDockerExecutor::new(LOAD_DATA_IMAGE)),
        })
        .invoke_handler(tauri::generate_handler![handle_load_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
