use axum::http::StatusCode;
use axum::Json;

/// Unified error type for the application.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] axum::http::Error),

    #[error("Axum error: {0}")]
    Axum(#[from] axum::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("All ports {0}-{1} are occupied")]
    PortConflict(u16, u16),

    #[error("State error: {0}")]
    State(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            Self::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Serialization(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Http(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Axum(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::PortConflict(base, end) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("All ports {base}-{end} are occupied"),
            ),
            Self::State(msg) => (StatusCode::CONFLICT, msg.clone()),
        };

        (status, Json(serde_json::json!({ "ok": false, "error": message }))).into_response()
    }
}