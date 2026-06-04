use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// Session states, matching the requirements document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionState {
    Idle,
    Working,
    NeedsInput,
    Error,
}

impl SessionState {
    /// Priority for aggregate state calculation.
    /// Error > NeedsInput > Working > Idle
    pub fn priority(self) -> u8 {
        match self {
            Self::Idle => 1,
            Self::Working => 2,
            Self::NeedsInput => 3,
            Self::Error => 4,
        }
    }

    pub fn from_priority(p: u8) -> Self {
        match p {
            4 => Self::Error,
            3 => Self::NeedsInput,
            2 => Self::Working,
            _ => Self::Idle,
        }
    }
}

/// Source of the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    ClaudeCode,
    Codex,
}

/// A single AI agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub label: String,
    pub state: SessionState,
    pub detail: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub source: SourceType,
}

/// Application state shared between HTTP server, tray, and frontend.
#[derive(Debug, Clone)]
pub struct AppState {
    pub sessions: HashMap<String, Session>,
    pub config: EngineConfig,
}

/// Engine configuration with defaults.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub session_ttl: Duration,
    pub needs_input_ttl: Duration,
    pub http_port: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            session_ttl: Duration::from_secs(30 * 60),        // 30 minutes
            needs_input_ttl: Duration::from_secs(2 * 60 * 60), // 2 hours
            http_port: 17321,
        }
    }
}

impl AppState {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            config,
        }
    }

    /// Update or create a session. Returns the new aggregate state.
    pub fn upsert_session(
        &mut self,
        session_id: String,
        state: SessionState,
        label: Option<String>,
        detail: Option<String>,
        source: SourceType,
    ) -> SessionState {
        let now = chrono::Utc::now();
        let entry = self.sessions.entry(session_id.clone()).or_insert_with(|| Session {
            id: session_id,
            label: label.clone().unwrap_or_else(|| "Session".to_string()),
            state,
            detail: detail.clone(),
            started_at: now,
            updated_at: now,
            source,
        });

        // Always accept the new state (lenient transition policy)
        let old_state = entry.state;
        if old_state != state {
            log::info!("Session {} transition: {old_state:?} -> {state:?}", entry.id);
        }
        entry.state = state;
        if let Some(lbl) = label {
            entry.label = lbl;
        }
        entry.detail = detail;
        entry.updated_at = now;

        aggregate_state(&self.sessions)
    }

    /// Remove a session. Returns the new aggregate state.
    pub fn remove_session(&mut self, session_id: &str) -> Option<SessionState> {
        if self.sessions.remove(session_id).is_some() {
            Some(aggregate_state(&self.sessions))
        } else {
            None
        }
    }

    /// Clean up sessions that have exceeded their TTL.
    /// Returns true if any sessions were removed.
    pub fn cleanup_expired(&mut self) -> bool {
        let now = chrono::Utc::now();
        let mut removed = false;
        self.sessions.retain(|_id, session| {
            let ttl = match session.state {
                SessionState::NeedsInput => self.config.needs_input_ttl,
                _ => self.config.session_ttl,
            };
            let age = now - session.updated_at;
            let ttl_delta = chrono::TimeDelta::from_std(ttl).unwrap_or(chrono::TimeDelta::MAX);
            let keep = age < ttl_delta;
            if !keep {
                log::info!("TTL expired for session {}", session.id);
                removed = true;
            }
            keep
        });
        removed
    }
}

// ---------------------------------------------------------------------------
// Aggregate state calculation
// ---------------------------------------------------------------------------

/// Compute the aggregate state across all sessions.
/// Returns Idle when no sessions exist.
pub fn aggregate_state(sessions: &HashMap<String, Session>) -> SessionState {
    sessions
        .values()
        .map(|s| s.state.priority())
        .max()
        .map(SessionState::from_priority)
        .unwrap_or(SessionState::Idle)
}

/// Convert a SessionState to its string representation for JSON.
pub fn state_to_string(state: &SessionState) -> String {
    match state {
        SessionState::Idle => "IDLE".to_string(),
        SessionState::Working => "WORKING".to_string(),
        SessionState::NeedsInput => "NEEDS_INPUT".to_string(),
        SessionState::Error => "ERROR".to_string(),
    }
}

pub fn aggregate_state_string(sessions: &HashMap<String, Session>) -> String {
    state_to_string(&aggregate_state(sessions))
}

// ---------------------------------------------------------------------------
// TTL cleanup background task
// ---------------------------------------------------------------------------

/// Background task that periodically cleans up expired sessions.
pub async fn start_ttl_cleanup(
    state: Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(5 * 60)); // Every 5 minutes
    loop {
        interval.tick().await;
        let mut app_state = state.write().await;
        if app_state.cleanup_expired() {
            let aggregate = aggregate_state(&app_state.sessions);
            let sessions_json = serde_json::to_value(&app_state.sessions).unwrap_or_default();
            drop(app_state);

            let _ = app.emit("state-updated", serde_json::json!({
                "sessions": sessions_json,
                "aggregate_state": state_to_string(&aggregate),
            }));

            // Write status file in background
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                let app_state = state_clone.read().await;
                if let Err(e) = crate::status_file::write_status_file(&app_state.sessions).await {
                    log::error!("Failed to write status file after TTL cleanup: {e}");
                }
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_state_empty() {
        let sessions = HashMap::new();
        assert_eq!(aggregate_state(&sessions), SessionState::Idle);
    }

    #[test]
    fn test_aggregate_state_single() {
        let mut sessions = HashMap::new();
        let now = chrono::Utc::now();
        sessions.insert("s1".to_string(), Session {
            id: "s1".to_string(),
            label: "Test".to_string(),
            state: SessionState::Working,
            detail: None,
            started_at: now,
            updated_at: now,
            source: SourceType::ClaudeCode,
        });
        assert_eq!(aggregate_state(&sessions), SessionState::Working);
    }

    #[test]
    fn test_aggregate_state_priority() {
        let mut sessions = HashMap::new();
        let now = chrono::Utc::now();

        sessions.insert("s1".to_string(), Session {
            id: "s1".to_string(), label: "A".to_string(),
            state: SessionState::Idle, detail: None,
            started_at: now, updated_at: now, source: SourceType::ClaudeCode,
        });
        sessions.insert("s2".to_string(), Session {
            id: "s2".to_string(), label: "B".to_string(),
            state: SessionState::NeedsInput, detail: None,
            started_at: now, updated_at: now, source: SourceType::ClaudeCode,
        });
        sessions.insert("s3".to_string(), Session {
            id: "s3".to_string(), label: "C".to_string(),
            state: SessionState::Working, detail: None,
            started_at: now, updated_at: now, source: SourceType::ClaudeCode,
        });

        // NeedsInput has highest priority
        assert_eq!(aggregate_state(&sessions), SessionState::NeedsInput);
    }

    #[test]
    fn test_aggregate_error_overrides_needs_input() {
        let mut sessions = HashMap::new();
        let now = chrono::Utc::now();

        sessions.insert("s1".to_string(), Session {
            id: "s1".to_string(), label: "A".to_string(),
            state: SessionState::Error, detail: None,
            started_at: now, updated_at: now, source: SourceType::ClaudeCode,
        });
        sessions.insert("s2".to_string(), Session {
            id: "s2".to_string(), label: "B".to_string(),
            state: SessionState::NeedsInput, detail: None,
            started_at: now, updated_at: now, source: SourceType::ClaudeCode,
        });

        // Error overrides NeedsInput
        assert_eq!(aggregate_state(&sessions), SessionState::Error);
    }

    #[test]
    fn test_upsert_and_remove() {
        let config = EngineConfig::default();
        let mut state = AppState::new(config);

        let agg = state.upsert_session(
            "s1".to_string(),
            SessionState::Working,
            Some("Bug Fix".to_string()),
            Some("Scanning codebase".to_string()),
            SourceType::ClaudeCode,
        );
        assert_eq!(agg, SessionState::Working);
        assert!(state.sessions.contains_key("s1"));

        let agg = state.remove_session("s1");
        assert!(agg.is_some());
        assert!(state.sessions.is_empty());
    }

    #[test]
    fn test_ttl_cleanup() {
        let config = EngineConfig {
            session_ttl: Duration::from_millis(100),
            needs_input_ttl: Duration::from_millis(500),
            ..EngineConfig::default()
        };
        let mut state = AppState::new(config);

        // Insert an Idle session
        state.upsert_session(
            "s1".to_string(),
            SessionState::Idle,
            Some("Old".to_string()),
            None,
            SourceType::ClaudeCode,
        );

        // Artificially age the session
        let now = chrono::Utc::now();
        if let Some(session) = state.sessions.get_mut("s1") {
            session.updated_at = now - chrono::Duration::seconds(200);
        }

        assert!(state.cleanup_expired());
        assert!(state.sessions.is_empty());
    }

    #[test]
    fn test_needs_input_longer_ttl() {
        let config = EngineConfig {
            session_ttl: Duration::from_millis(100),
            needs_input_ttl: Duration::from_millis(500),
            ..EngineConfig::default()
        };
        let mut state = AppState::new(config);

        state.upsert_session(
            "s1".to_string(),
            SessionState::NeedsInput,
            Some("Waiting".to_string()),
            None,
            SourceType::ClaudeCode,
        );

        // Session was updated 200ms ago (past session_ttl but within needs_input_ttl)
        let now = chrono::Utc::now();
        if let Some(session) = state.sessions.get_mut("s1") {
            session.updated_at = now - chrono::Duration::milliseconds(200);
        }

        assert!(!state.cleanup_expired());
        assert!(state.sessions.contains_key("s1"));
    }
}