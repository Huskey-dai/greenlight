use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::RwLock;

mod error;
mod http;
mod icons;
mod state;
mod status_file;
mod theme;
mod tray;

pub use error::AppError;

/// Run the Greenlight application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Create shared application state
            let app_state = Arc::new(RwLock::new(state::AppState::new(state::EngineConfig::default())));

            // Start the HTTP server on a background task
            let state_clone = app_state.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = http::start_server(state_clone, &handle).await {
                    log::error!("HTTP server error: {e}");
                }
            });

            // Start TTL cleanup task
            let state_clone = app_state.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                state::start_ttl_cleanup(state_clone, &handle).await;
            });

            // Setup system tray
            tray::setup_tray(app, app_state.clone())?;

            // Restore sessions from status.json if it exists
            let state_clone = app_state.clone();
            tauri::async_runtime::block_on(async {
                let mut state = state_clone.write().await;
                if let Ok(sessions) = status_file::read_status_file().await {
                    state.sessions = sessions;
                    log::info!("Restored {} sessions from status.json", state.sessions.len());
                }
            });

            // Register Tauri commands
            app.manage(app_state.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_aggregate_state,
            remove_session,
            rename_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Greenlight application");
}

// --- Tauri command handlers ---

#[tauri::command]
async fn get_sessions(
    state: tauri::State<'_, Arc<RwLock<state::AppState>>>,
) -> Result<serde_json::Value, String> {
    let app_state = state.read().await;
    serde_json::to_value(&app_state.sessions).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_aggregate_state(
    state: tauri::State<'_, Arc<RwLock<state::AppState>>>,
) -> Result<String, String> {
    let app_state = state.read().await;
    Ok(state::aggregate_state_string(&app_state.sessions))
}

#[tauri::command]
async fn remove_session(
    session_id: String,
    state: tauri::State<'_, Arc<RwLock<state::AppState>>>,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let mut app_state = state.write().await;
    if app_state.sessions.remove(&session_id).is_some() {
        let aggregate = state::aggregate_state(&app_state.sessions);
        drop(app_state);
        let _ = app.emit("state-updated", serde_json::json!({
            "sessions": {},
            "aggregate_state": state::state_to_string(&aggregate),
        }));
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn rename_session(
    session_id: String,
    label: String,
    state: tauri::State<'_, Arc<RwLock<state::AppState>>>,
) -> Result<bool, String> {
    let label_trimmed = label.trim().to_string();
    if label_trimmed.is_empty() || label_trimmed.len() > 50 {
        return Err("Label must be 1-50 characters".to_string());
    }
    let mut app_state = state.write().await;
    if let Some(session) = app_state.sessions.get_mut(&session_id) {
        session.label = label_trimmed;
        Ok(true)
    } else {
        Ok(false)
    }
}