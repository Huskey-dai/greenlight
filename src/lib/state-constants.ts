// State type definitions shared between Rust and TypeScript

export type SessionState = 'IDLE' | 'WORKING' | 'NEEDS_INPUT' | 'ERROR';
export type DisplayState = SessionState | 'UNKNOWN';
export type SourceType = 'claude_code' | 'codex';
export type EventSourceType = SourceType | 'detected_process' | 'system';

export interface Session {
  id: string;
  label: string;
  state: SessionState;
  detail?: string;
  started_at: string;
  updated_at: string;
  source: SourceType;
}

export interface SessionsPayload {
  sessions: Record<string, Session>;
  aggregate_state: string;
  recent_events?: StateEvent[];
}

export interface DiagnosticFile {
  label: string;
  path: string;
  exists: boolean;
  status: 'ok' | 'missing';
}

export interface StateEvent {
  id: string;
  timestamp: string;
  session_id: string;
  label: string;
  from_state?: SessionState;
  to_state?: SessionState;
  source: EventSourceType;
  detail?: string;
}

export interface HookInstallResult {
  ok: boolean;
  hooks_dir: string;
  installed_count: number;
  hook_files: DiagnosticFile[];
}

export interface Diagnostics {
  version: string;
  http_port?: number;
  port_file: string;
  port_file_exists: boolean;
  status_file: string;
  status_file_exists: boolean;
  process_monitor_enabled: boolean;
  sessions_count: number;
  process_sessions_count: number;
  by_state: Record<string, number>;
  by_source: Record<string, number>;
  latest_update?: string;
  hooks_ready: boolean;
  hook_files: DiagnosticFile[];
}

export const STATE_COLORS: Record<DisplayState, string> = {
  IDLE: '#22C55E',
  WORKING: '#EAB308',
  NEEDS_INPUT: '#EF4444',
  ERROR: '#EF4444',
  UNKNOWN: '#6B7280',
} as const;

export const STATE_LABELS: Record<DisplayState, string> = {
  IDLE: '\u7a7a\u95f2',
  WORKING: '\u5de5\u4f5c\u4e2d',
  NEEDS_INPUT: '\u9700\u6388\u6743',
  ERROR: '\u51fa\u9519',
  UNKNOWN: '\u672a\u8fde\u63a5',
} as const;

export const STATE_SUMMARIES: Record<DisplayState, string> = {
  IDLE: '\u5f53\u524d\u6ca1\u6709\u9700\u8981\u5904\u7406\u7684\u4efb\u52a1',
  WORKING: '\u6b63\u5728\u6267\u884c\u4efb\u52a1',
  NEEDS_INPUT: '\u7b49\u5f85\u4f60\u786e\u8ba4\u6216\u6388\u6743',
  ERROR: '\u4efb\u52a1\u9047\u5230\u95ee\u9898',
  UNKNOWN: '\u72b6\u6001\u5f15\u64ce\u5c1a\u672a\u8fde\u63a5',
} as const;

export const SOURCE_LABELS: Record<SourceType, string> = {
  claude_code: 'Claude Code',
  codex: 'Codex \u5ba2\u6237\u7aef',
} as const;

export const BLINK_CONFIG = {
  WORKING: { frequency: 1, dutyCycle: 0.5 },
  NEEDS_INPUT: { frequency: 2, dutyCycle: 0.3 },
} as const;
