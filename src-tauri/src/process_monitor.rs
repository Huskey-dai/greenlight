use crate::state::{self, AppState, SessionState, SourceType};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;

const PROCESS_SESSION_PREFIX: &str = "process-";
const POLL_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliKind {
    Codex,
    ClaudeCode,
}

impl CliKind {
    fn id_prefix(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex \u{5ba2}\u{6237}\u{7aef}",
            Self::ClaudeCode => "Claude Code",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::Codex => "\u{5ba2}\u{6237}\u{7aef}\u{5df2}\u{8fde}\u{63a5}",
            Self::ClaudeCode => "CLI running",
        }
    }

    fn default_state(self) -> SessionState {
        SessionState::Idle
    }

    fn source(self) -> SourceType {
        match self {
            Self::Codex => SourceType::Codex,
            Self::ClaudeCode => SourceType::ClaudeCode,
        }
    }
}

#[derive(Debug, Clone)]
struct ProcessRecord {
    pid: u32,
    parent_pid: Option<u32>,
    name: String,
    command_line: String,
}

#[derive(Debug, Clone)]
struct DiscoveredCliSession {
    id: String,
    label: String,
    detail: String,
    state: SessionState,
    source: SourceType,
}

pub async fn start_cli_process_monitor(
    shared_state: Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
) {
    let app = app.clone();

    loop {
        if let Err(e) = sync_cli_processes(shared_state.clone(), &app).await {
            log::debug!("CLI process monitor skipped a scan: {e}");
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn sync_cli_processes(
    shared_state: Arc<RwLock<AppState>>,
    app: &tauri::AppHandle,
) -> Result<(), String> {
    let discovered = discover_cli_sessions()?;
    let detected_ids: HashSet<String> = discovered
        .iter()
        .map(|session| session.id.clone())
        .collect();

    let mut app_state = shared_state.write().await;
    let mut changed = false;
    let now = chrono::Utc::now();

    for discovered_session in discovered {
        if let Some(existing) = app_state.sessions.get_mut(&discovered_session.id) {
            changed |= update_detected_process_session(existing, &discovered_session, now);
        } else {
            app_state.upsert_session(
                discovered_session.id,
                discovered_session.state,
                Some(discovered_session.label),
                Some(discovered_session.detail),
                discovered_session.source,
            );
            changed = true;
        }
    }

    let stale_ids: Vec<String> = app_state
        .sessions
        .iter()
        .filter(|(id, session)| is_stale_detected_process_session(id, session, &detected_ids))
        .map(|(id, _session)| id.clone())
        .collect();

    for id in stale_ids {
        app_state.sessions.remove(&id);
        changed = true;
    }

    if !changed {
        return Ok(());
    }

    let aggregate = state::aggregate_state(&app_state.sessions);
    let sessions_json = serde_json::to_value(&app_state.sessions).unwrap_or_default();
    let sessions_clone = app_state.sessions.clone();
    drop(app_state);

    let _ = app.emit(
        "state-updated",
        serde_json::json!({
            "sessions": sessions_json,
            "aggregate_state": state::state_to_string(&aggregate),
        }),
    );

    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::status_file::write_status_file(&sessions_clone).await {
            log::error!("Failed to write status file after CLI process scan: {e}");
        }
    });

    Ok(())
}

fn update_detected_process_session(
    existing: &mut state::Session,
    discovered_session: &DiscoveredCliSession,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    let mut changed = false;
    let owns_detail = is_detected_process_detail(existing.detail.as_deref());

    existing.updated_at = now;
    if existing.label != discovered_session.label {
        existing.label = discovered_session.label.clone();
        changed = true;
    }

    if existing.state == SessionState::Working && owns_detail {
        existing.state = discovered_session.state;
        changed = true;
    }

    if existing.detail.is_none() || owns_detail {
        if existing.detail.as_deref() != Some(discovered_session.detail.as_str()) {
            existing.detail = Some(discovered_session.detail.clone());
            changed = true;
        }
    }

    changed
}

fn is_stale_detected_process_session(
    id: &str,
    session: &state::Session,
    detected_ids: &HashSet<String>,
) -> bool {
    id.starts_with(PROCESS_SESSION_PREFIX)
        && !detected_ids.contains(id)
        && is_detected_process_detail(session.detail.as_deref())
}

fn discover_cli_sessions() -> Result<Vec<DiscoveredCliSession>, String> {
    let current_pid = std::process::id();
    let processes = platform_processes()?;
    Ok(discover_cli_sessions_from_processes(
        &processes,
        current_pid,
    ))
}

fn discover_cli_sessions_from_processes(
    processes: &[ProcessRecord],
    current_pid: u32,
) -> Vec<DiscoveredCliSession> {
    let classified_processes: Vec<(&ProcessRecord, CliKind)> = processes
        .iter()
        .filter(|process| process.pid != current_pid)
        .filter_map(|process| {
            classify_process(&process.name, &process.command_line).map(|kind| (process, kind))
        })
        .collect();
    let classified_pids: HashSet<u32> = classified_processes
        .iter()
        .map(|(process, _kind)| process.pid)
        .collect();
    let parent_by_pid: HashMap<u32, u32> = processes
        .iter()
        .filter_map(|process| {
            process
                .parent_pid
                .map(|parent_pid| (process.pid, parent_pid))
        })
        .collect();
    let mut seen = HashSet::new();
    let mut sessions = Vec::new();

    for (process, kind) in classified_processes {
        if has_classified_ancestor(process, &parent_by_pid, &classified_pids) {
            continue;
        }

        let id = format!(
            "{PROCESS_SESSION_PREFIX}{}-{}",
            kind.id_prefix(),
            process.pid
        );
        if !seen.insert(id.clone()) {
            continue;
        }

        sessions.push(DiscoveredCliSession {
            id,
            label: kind.label().to_string(),
            detail: kind.detail().to_string(),
            state: kind.default_state(),
            source: kind.source(),
        });
    }

    sessions
}

fn is_detected_process_detail(detail: Option<&str>) -> bool {
    matches!(
        detail,
        Some("CLI running")
            | Some("\u{5ba2}\u{6237}\u{7aef}\u{8fd0}\u{884c}\u{4e2d}")
            | Some("\u{5ba2}\u{6237}\u{7aef}\u{5df2}\u{8fde}\u{63a5}")
    )
}

fn has_classified_ancestor(
    process: &ProcessRecord,
    parent_by_pid: &HashMap<u32, u32>,
    classified_pids: &HashSet<u32>,
) -> bool {
    let mut visited = HashSet::new();
    let mut current = process.parent_pid;

    while let Some(pid) = current {
        if classified_pids.contains(&pid) {
            return true;
        }
        if !visited.insert(pid) {
            return false;
        }
        current = parent_by_pid.get(&pid).copied();
    }

    false
}

#[cfg(windows)]
fn platform_processes() -> Result<Vec<ProcessRecord>, String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-CimInstance Win32_Process |
  Where-Object {
    $_.Name -match '^(codex|claude|cc|node|bun|deno)(\.exe)?$' -or
    $_.CommandLine -match '(codex|claude|cc)'
  } |
  Select-Object ProcessId,ParentProcessId,Name,CommandLine |
  ConvertTo-Json -Compress
"#;

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("failed to run powershell process scan: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "powershell process scan exited with {}",
            output.status
        ));
    }

    parse_windows_process_json(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn parse_windows_process_json(output: &str) -> Result<Vec<ProcessRecord>, String> {
    let output = output.trim();
    if output.is_empty() {
        return Ok(Vec::new());
    }

    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| format!("failed to parse process scan json: {e}"))?;

    let records = match value {
        serde_json::Value::Array(items) => items.iter().filter_map(parse_windows_process).collect(),
        serde_json::Value::Object(_) => parse_windows_process(&value).into_iter().collect(),
        _ => Vec::new(),
    };

    Ok(records)
}

#[cfg(windows)]
fn parse_windows_process(value: &serde_json::Value) -> Option<ProcessRecord> {
    let pid = value.get("ProcessId")?.as_u64()? as u32;
    let parent_pid = value
        .get("ParentProcessId")
        .and_then(|v| v.as_u64())
        .map(|pid| pid as u32);
    let name = value
        .get("Name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let command_line = value
        .get("CommandLine")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Some(ProcessRecord {
        pid,
        parent_pid,
        name,
        command_line,
    })
}

#[cfg(not(windows))]
fn platform_processes() -> Result<Vec<ProcessRecord>, String> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,ppid=,comm=,args="])
        .output()
        .map_err(|e| format!("failed to run ps process scan: {e}"))?;

    if !output.status.success() {
        return Err(format!("ps process scan exited with {}", output.status));
    }

    Ok(parse_ps_output(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(not(windows))]
fn parse_ps_output(output: &str) -> Vec<ProcessRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid, rest) = split_once_whitespace(line)?;
            let pid = pid.parse::<u32>().ok()?;
            let (parent_pid, rest) = split_once_whitespace(rest)?;
            let parent_pid = parent_pid.parse::<u32>().ok();
            let (name, command_line) = split_once_whitespace(rest).unwrap_or((rest, ""));
            Some(ProcessRecord {
                pid,
                parent_pid,
                name: name.to_string(),
                command_line: command_line.to_string(),
            })
        })
        .collect()
}

#[cfg(not(windows))]
fn split_once_whitespace(value: &str) -> Option<(&str, &str)> {
    let split_at = value.find(char::is_whitespace)?;
    let first = &value[..split_at];
    let rest = value[split_at..].trim_start();
    Some((first, rest))
}

fn classify_process(name: &str, command_line: &str) -> Option<CliKind> {
    classify_token(name).or_else(|| {
        command_line
            .split(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\'')
            .find_map(classify_token)
    })
}

fn classify_token(token: &str) -> Option<CliKind> {
    let token = token
        .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == '`' || ch == ',' || ch == ';');
    if token.is_empty() {
        return None;
    }

    let lower = token.to_ascii_lowercase();
    let normalized = lower.replace('\\', "/");
    if normalized.contains("@openai/codex/bin/codex")
        || normalized.contains("openai-codex/bin/codex")
    {
        return Some(CliKind::Codex);
    }
    if normalized.contains("@anthropic-ai/claude-code/cli") {
        return Some(CliKind::ClaudeCode);
    }

    match executable_stem(&lower).as_str() {
        "codex" => Some(CliKind::Codex),
        "claude" | "claude-code" | "cc" => Some(CliKind::ClaudeCode),
        _ => None,
    }
}

fn executable_stem(token: &str) -> String {
    let basename = token
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(token)
        .trim_end_matches([')', ']']);

    for ext in [".exe", ".cmd", ".ps1", ".bat", ".js"] {
        if let Some(stripped) = basename.strip_suffix(ext) {
            return stripped.to_string();
        }
    }

    basename.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn discovered_codex_session(id: &str) -> DiscoveredCliSession {
        DiscoveredCliSession {
            id: id.to_string(),
            label: "Codex \u{5ba2}\u{6237}\u{7aef}".to_string(),
            detail: "\u{5ba2}\u{6237}\u{7aef}\u{5df2}\u{8fde}\u{63a5}".to_string(),
            state: SessionState::Idle,
            source: SourceType::Codex,
        }
    }

    fn session(id: &str, state: SessionState, detail: Option<&str>) -> state::Session {
        let now = chrono::Utc::now();
        state::Session {
            id: id.to_string(),
            label: "Codex \u{5ba2}\u{6237}\u{7aef}".to_string(),
            state,
            detail: detail.map(str::to_string),
            started_at: now,
            updated_at: now,
            source: SourceType::Codex,
        }
    }

    #[test]
    fn classifies_codex_binary_name() {
        assert_eq!(classify_process("codex.exe", ""), Some(CliKind::Codex));
    }

    #[test]
    fn classifies_node_hosted_claude_code() {
        let command_line =
            r#"node C:\Users\me\AppData\Roaming\npm\node_modules\@anthropic-ai\claude-code\cli.js"#;
        assert_eq!(
            classify_process("node.exe", command_line),
            Some(CliKind::ClaudeCode)
        );
    }

    #[test]
    fn classifies_node_hosted_codex() {
        let command_line = r#"node /usr/local/lib/node_modules/@openai/codex/bin/codex.js"#;
        assert_eq!(classify_process("node", command_line), Some(CliKind::Codex));
    }

    #[test]
    fn ignores_codex_worker_processes() {
        let command_line = r#"node C:\npm\node_modules\@openai\codex\worker.js"#;
        assert_eq!(classify_process("node.exe", command_line), None);
    }

    #[test]
    fn ignores_incidental_codex_path_text() {
        let command_line = r#"node C:\Users\CodexSandboxOffline\app\server.js"#;
        assert_eq!(classify_process("node.exe", command_line), None);
    }

    #[test]
    fn keeps_only_top_level_classified_processes() {
        let processes = vec![
            ProcessRecord {
                pid: 10,
                parent_pid: Some(1),
                name: "node.exe".to_string(),
                command_line: r#"node C:\npm\node_modules\@openai\codex\bin\codex.js"#.to_string(),
            },
            ProcessRecord {
                pid: 11,
                parent_pid: Some(10),
                name: "node.exe".to_string(),
                command_line: r#"node C:\npm\node_modules\@openai\codex\worker.js"#.to_string(),
            },
            ProcessRecord {
                pid: 12,
                parent_pid: Some(11),
                name: "codex.exe".to_string(),
                command_line: "codex task child".to_string(),
            },
        ];

        let sessions = discover_cli_sessions_from_processes(&processes, 999);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "process-codex-10");
        assert_eq!(sessions[0].label, "Codex \u{5ba2}\u{6237}\u{7aef}");
        assert_eq!(sessions[0].state, SessionState::Idle);
    }

    #[test]
    fn downgrades_legacy_detected_working_placeholder_to_idle() {
        let mut existing = session(
            "process-codex-10",
            SessionState::Working,
            Some("\u{5ba2}\u{6237}\u{7aef}\u{8fd0}\u{884c}\u{4e2d}"),
        );
        let discovered = discovered_codex_session("process-codex-10");

        let changed =
            update_detected_process_session(&mut existing, &discovered, chrono::Utc::now());

        assert!(changed);
        assert_eq!(existing.state, SessionState::Idle);
        assert_eq!(
            existing.detail.as_deref(),
            Some("\u{5ba2}\u{6237}\u{7aef}\u{5df2}\u{8fde}\u{63a5}")
        );
    }

    #[test]
    fn preserves_authoritative_needs_input_session() {
        let mut existing = session(
            "process-codex-10",
            SessionState::NeedsInput,
            Some("Permission: run command"),
        );
        let discovered = discovered_codex_session("process-codex-10");

        update_detected_process_session(&mut existing, &discovered, chrono::Utc::now());

        assert_eq!(existing.state, SessionState::NeedsInput);
        assert_eq!(existing.detail.as_deref(), Some("Permission: run command"));
    }

    #[test]
    fn preserves_authoritative_working_session() {
        let mut existing = session(
            "process-codex-10",
            SessionState::Working,
            Some("Tool: shell_command"),
        );
        let discovered = discovered_codex_session("process-codex-10");

        update_detected_process_session(&mut existing, &discovered, chrono::Utc::now());

        assert_eq!(existing.state, SessionState::Working);
        assert_eq!(existing.detail.as_deref(), Some("Tool: shell_command"));
    }

    #[test]
    fn removes_only_stale_detected_process_placeholders() {
        let detected_ids = HashSet::new();
        let placeholder = session(
            "process-codex-10",
            SessionState::Idle,
            Some("\u{5ba2}\u{6237}\u{7aef}\u{5df2}\u{8fde}\u{63a5}"),
        );
        let hook_session = session(
            "process-codex-11",
            SessionState::NeedsInput,
            Some("Permission: run command"),
        );

        assert!(is_stale_detected_process_session(
            "process-codex-10",
            &placeholder,
            &detected_ids,
        ));
        assert!(!is_stale_detected_process_session(
            "process-codex-11",
            &hook_session,
            &detected_ids,
        ));
    }
}
