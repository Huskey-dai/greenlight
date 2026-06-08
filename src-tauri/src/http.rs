use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::state::{self, AppState, SessionState, SourceType};
use crate::status_file;

/// Shared state type for axum handlers.
type SharedAppState = Arc<RwLock<AppState>>;

#[derive(Clone)]
struct HttpServerState {
    app_state: SharedAppState,
    app_handle: tauri::AppHandle,
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UpdateStateRequest {
    pub state: SessionState,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default = "default_source")]
    pub source: SourceType,
}

fn default_source() -> SourceType {
    SourceType::ClaudeCode
}

#[derive(Debug, serde::Serialize)]
pub struct ApiResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub sessions_count: usize,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn update_session_state(
    State(server_state): State<HttpServerState>,
    Path(session_id): Path<String>,
    Json(body): Json<UpdateStateRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), AppError> {
    // Validate session_id format
    let id_re = regex::Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap();
    if !id_re.is_match(&session_id) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                ok: false,
                session_id: None,
                aggregate_state: None,
                error: Some("Invalid session_id: must match ^[a-zA-Z0-9_-]{1,64}$".to_string()),
            }),
        ));
    }

    // Validate label length
    if let Some(ref label) = body.label {
        let trimmed = label.trim();
        if trimmed.is_empty() || trimmed.len() > 50 {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    ok: false,
                    session_id: None,
                    aggregate_state: None,
                    error: Some("Label must be 1-50 characters".to_string()),
                }),
            ));
        }
    }

    // Validate detail length
    if let Some(ref detail) = body.detail {
        if detail.len() > 200 {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    ok: false,
                    session_id: None,
                    aggregate_state: None,
                    error: Some("Detail must be 0-200 characters".to_string()),
                }),
            ));
        }
    }

    let mut app_state = server_state.app_state.write().await;
    let previous = app_state.sessions.get(&session_id).cloned();
    let aggregate = app_state.upsert_session(
        session_id.clone(),
        body.state,
        body.label.clone(),
        body.detail.clone(),
        body.source.clone(),
    );
    let current = app_state.sessions.get(&session_id).cloned();
    if previous.as_ref().is_none_or(|session| {
        session.state != body.state || session.detail != body.detail
    }) {
        if let Some(session) = current {
            app_state.record_event(
                session_id.clone(),
                session.label.clone(),
                previous.map(|session| session.state),
                Some(session.state),
                state::source_to_string(&session.source),
                session.detail.clone(),
            );
        }
    }

    let sessions_json = serde_json::to_value(&app_state.sessions).unwrap_or_default();
    let events_json = serde_json::to_value(&app_state.recent_events).unwrap_or_default();

    // Write status file in background (non-blocking)
    let sessions_clone = app_state.sessions.clone();
    drop(app_state);

    emit_state_updated(
        &server_state.app_handle,
        sessions_json,
        events_json,
        &aggregate,
    );

    tauri::async_runtime::spawn(async move {
        if let Err(e) = status_file::write_status_file(&sessions_clone).await {
            log::error!("Failed to write status file: {e}");
        }
    });

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            ok: true,
            session_id: Some(session_id),
            aggregate_state: Some(state::state_to_string(&aggregate)),
            error: None,
        }),
    ))
}

async fn remove_session(
    State(server_state): State<HttpServerState>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse>, AppError> {
    let mut app_state = server_state.app_state.write().await;
    let removed = app_state.sessions.get(&session_id).cloned();
    if let Some(aggregate) = app_state.remove_session(&session_id) {
        if let Some(session) = removed {
            app_state.record_event(
                session_id.clone(),
                session.label.clone(),
                Some(session.state),
                None,
                "system",
                Some("Session removed".to_string()),
            );
        }
        let sessions_json = serde_json::to_value(&app_state.sessions).unwrap_or_default();
        let events_json = serde_json::to_value(&app_state.recent_events).unwrap_or_default();
        let sessions_clone = app_state.sessions.clone();
        drop(app_state);

        emit_state_updated(
            &server_state.app_handle,
            sessions_json,
            events_json,
            &aggregate,
        );

        tauri::async_runtime::spawn(async move {
            if let Err(e) = status_file::write_status_file(&sessions_clone).await {
                log::error!("Failed to write status file after delete: {e}");
            }
        });

        Ok(Json(ApiResponse {
            ok: true,
            session_id: Some(session_id),
            aggregate_state: Some(state::state_to_string(&aggregate)),
            error: None,
        }))
    } else {
        Ok(Json(ApiResponse {
            ok: true,
            session_id: Some(session_id),
            aggregate_state: Some(state::state_to_string(&state::aggregate_state(
                &server_state.app_state.read().await.sessions,
            ))),
            error: None,
        }))
    }
}

async fn list_sessions(
    State(server_state): State<HttpServerState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let app_state = server_state.app_state.read().await;
    Ok(Json(serde_json::to_value(&app_state.sessions)?))
}

async fn health_check(State(server_state): State<HttpServerState>) -> Json<HealthResponse> {
    let app_state = server_state.app_state.read().await;
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        sessions_count: app_state.sessions.len(),
    })
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

/// Start the HTTP server. Tries ports 17321-17331, writes the selected port to port.txt.
pub async fn start_server(
    shared_state: Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
) -> Result<(), AppError> {
    let base_port = shared_state.read().await.config.http_port;
    let server_state = HttpServerState {
        app_state: shared_state.clone(),
        app_handle: app.clone(),
    };

    let (selected_port, server) = 'port_search: {
        for port in base_port..=(base_port + 10) {
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(_) => continue,
            };
            break 'port_search (port, listener);
        }
        return Err(AppError::PortConflict(base_port, base_port + 10));
    };

    log::info!("HTTP server listening on 127.0.0.1:{selected_port}");

    // Write port file for hook scripts
    if let Err(e) = write_port_file(selected_port).await {
        log::warn!("Failed to write port file: {e}");
    }

    let app = Router::new()
        .route(
            "/api/sessions/{session_id}/state",
            post(update_session_state),
        )
        .route("/api/sessions/{session_id}", delete(remove_session))
        .route("/api/sessions", get(list_sessions))
        .route("/api/health", get(health_check))
        .with_state(server_state);

    // Graceful shutdown: delete port file on server stop
    let port_file_cleanup = async {
        let _ = delete_port_file().await;
    };

    axum::serve(server, app)
        .with_graceful_shutdown(async {
            // Wait forever unless the Tauri app shuts down
            std::future::pending::<()>().await;
            port_file_cleanup.await;
        })
        .await
        .map_err(AppError::from)
}

fn emit_state_updated(
    app: &tauri::AppHandle,
    sessions_json: serde_json::Value,
    events_json: serde_json::Value,
    aggregate: &SessionState,
) {
    let _ = app.emit(
        "state-updated",
        serde_json::json!({
            "sessions": sessions_json,
            "aggregate_state": state::state_to_string(aggregate),
            "recent_events": events_json,
        }),
    );
}

async fn write_port_file(port: u16) -> std::io::Result<()> {
    let greenlight_dir = dirs::greenlight_dir()?;
    std::fs::create_dir_all(&greenlight_dir)?;
    let port_path = greenlight_dir.join("port.txt");
    std::fs::write(&port_path, port.to_string())
}

async fn delete_port_file() -> std::io::Result<()> {
    let port_path = dirs::greenlight_dir()?.join("port.txt");
    if port_path.exists() {
        std::fs::remove_file(port_path)
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helper module for directory paths
// ---------------------------------------------------------------------------

mod dirs {
    use std::path::PathBuf;

    pub fn greenlight_dir() -> std::io::Result<PathBuf> {
        let home = dirs_home()?;
        Ok(home.join(".greenlight"))
    }

    fn dirs_home() -> std::io::Result<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find home directory")
            })
    }
}
