use crate::state::{Session, SourceType, SessionState};
use std::collections::HashMap;
use std::path::PathBuf;

/// Atomically write the status.json file.
///
/// Algorithm:
/// 1. Serialize sessions to JSON
/// 2. Write to status.json.tmp
/// 3. Rename status.json.tmp -> status.json (atomic on same filesystem)
/// 4. Set file permissions to 0600 on Unix
pub async fn write_status_file(sessions: &HashMap<String, Session>) -> std::io::Result<()> {
    let greenlight_dir = greenlight_dir()?;
    std::fs::create_dir_all(&greenlight_dir)?;

    let status_path = greenlight_dir.join("status.json");
    let tmp_path = greenlight_dir.join("status.json.tmp");

    let snapshot = StatusSnapshot {
        version: 1,
        timestamp: chrono::Utc::now().to_rfc3339(),
        sessions: serde_json::to_value(sessions)?
            .as_object()
            .cloned()
            .unwrap_or_default(),
        meta: Some(StatusMeta {
            poll_interval_ms: 1000,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    };

    let json = serde_json::to_string_pretty(&snapshot)?;

    // Write to temp file first
    std::fs::write(&tmp_path, &json)?;

    // Atomic rename
    match std::fs::rename(&tmp_path, &status_path) {
        Ok(()) => {}
        Err(e) => {
            // On Windows, rename may fail if the target file is being read.
            // Fallback: delete target first, then rename.
            if cfg!(target_os = "windows") {
                let _ = std::fs::remove_file(&status_path);
                std::fs::rename(&tmp_path, &status_path)?;
            } else {
                return Err(e);
            }
        }
    }

    // Set permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&status_path, perms)?;
    }

    Ok(())
}

/// Read the status.json file to restore sessions on startup.
pub async fn read_status_file() -> std::io::Result<HashMap<String, Session>> {
    let greenlight_dir = greenlight_dir()?;
    let status_path = greenlight_dir.join("status.json");

    if !status_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&status_path)?;
    let snapshot: StatusSnapshot = serde_json::from_str(&content)?;

    let mut sessions = HashMap::new();
    for (id, value) in snapshot.sessions {
        // Parse each session from JSON value
        if let Some(session) = parse_session(&id, &value) {
            sessions.insert(id, session);
        }
    }

    Ok(sessions)
}

fn parse_session(id: &str, value: &serde_json::Value) -> Option<Session> {
    let obj = value.as_object()?;
    let state_str = obj.get("state")?.as_str()?;
    let state = match state_str {
        "IDLE" => SessionState::Idle,
        "WORKING" => SessionState::Working,
        "NEEDS_INPUT" => SessionState::NeedsInput,
        "ERROR" => SessionState::Error,
        _ => return None,
    };

    let label = obj.get("label")?.as_str()?.to_string();
    let detail = obj.get("detail").and_then(|v| v.as_str()).map(String::from);

    let started_at_str = obj
        .get("started_at")
        .or_else(|| obj.get("startedAt"))
        .and_then(|v| v.as_str());
    let started_at = started_at_str
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let updated_at_str = obj
        .get("updated_at")
        .or_else(|| obj.get("updatedAt"))
        .and_then(|v| v.as_str());
    let updated_at = updated_at_str
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);

    let source_str = obj.get("source").and_then(|v| v.as_str()).unwrap_or("claude_code");
    let source = match source_str {
        "codex" => SourceType::Codex,
        _ => SourceType::ClaudeCode,
    };

    Some(Session {
        id: id.to_string(),
        label,
        state,
        detail,
        started_at,
        updated_at,
        source,
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StatusSnapshot {
    version: u32,
    timestamp: String,
    sessions: serde_json::Map<String, serde_json::Value>,
    meta: Option<StatusMeta>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StatusMeta {
    poll_interval_ms: u32,
    engine_version: String,
}

fn greenlight_dir() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find home directory"))?;
    Ok(PathBuf::from(home).join(".greenlight"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_accepts_snake_case_timestamps() {
        let value = serde_json::json!({
            "state": "WORKING",
            "label": "Codex",
            "detail": "Using shell",
            "started_at": "2026-06-05T01:02:03Z",
            "updated_at": "2026-06-05T01:03:04Z",
            "source": "codex"
        });

        let session = parse_session("s1", &value).expect("session should parse");

        assert_eq!(session.id, "s1");
        assert_eq!(session.label, "Codex");
        assert_eq!(session.state, SessionState::Working);
        assert_eq!(session.started_at.to_rfc3339(), "2026-06-05T01:02:03+00:00");
        assert_eq!(session.updated_at.to_rfc3339(), "2026-06-05T01:03:04+00:00");
        assert!(matches!(session.source, SourceType::Codex));
    }

    #[test]
    fn parse_session_accepts_camel_case_timestamps() {
        let value = serde_json::json!({
            "state": "NEEDS_INPUT",
            "label": "Claude",
            "startedAt": "2026-06-05T02:02:03Z",
            "updatedAt": "2026-06-05T02:03:04Z",
            "source": "claude_code"
        });

        let session = parse_session("s2", &value).expect("session should parse");

        assert_eq!(session.id, "s2");
        assert_eq!(session.state, SessionState::NeedsInput);
        assert_eq!(session.started_at.to_rfc3339(), "2026-06-05T02:02:03+00:00");
        assert_eq!(session.updated_at.to_rfc3339(), "2026-06-05T02:03:04+00:00");
        assert!(matches!(session.source, SourceType::ClaudeCode));
    }
}
