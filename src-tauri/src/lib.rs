use std::sync::Arc;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::RwLock;

mod error;
mod http;
mod icons;
mod process_monitor;
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

            // Create the popup window (hidden by default, shown on tray click)
            let _popup = tauri::WebviewWindowBuilder::new(
                app,
                "popup",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Greenlight")
            .inner_size(360.0, 500.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false) // Hidden by default — toggled via tray icon
            .build()?;

            // Restore sessions from status.json if it exists
            let state_clone = app_state.clone();
            tauri::async_runtime::block_on(async {
                let mut state = state_clone.write().await;
                if let Ok(sessions) = status_file::read_status_file().await {
                    state.sessions = sessions;
                    log::info!("Restored {} sessions from status.json", state.sessions.len());
                }
            });

            // Start the HTTP server on a background task after restoring state.
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

            // Detect Codex / Claude Code CLI processes even before hooks emit a state.
            let state_clone = app_state.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                process_monitor::start_cli_process_monitor(state_clone, &handle).await;
            });

            // Setup system tray (after popup window is created and state is restored)
            tray::setup_tray(app, app_state.clone())?;

            // Start theme listener
            theme::start_theme_listener(app.handle());

            // Register Tauri command handlers
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
        let sessions_json = serde_json::to_value(&app_state.sessions).unwrap_or_default();
        drop(app_state);

        let _ = app.emit("state-updated", serde_json::json!({
            "sessions": sessions_json,
            "aggregate_state": state::state_to_string(&aggregate),
        }));

        // Write status file in background
        let state_clone = state.inner().clone();
        tauri::async_runtime::spawn(async move {
            let app_state = state_clone.read().await;
            if let Err(e) = status_file::write_status_file(&app_state.sessions).await {
                log::error!("Failed to write status file after remove: {e}");
            }
        });

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
