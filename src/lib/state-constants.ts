// State type definitions shared between Rust and TypeScript

export type SessionState = 'IDLE' | 'WORKING' | 'NEEDS_INPUT' | 'ERROR';
export type DisplayState = SessionState | 'UNKNOWN';
export type SourceType = 'claude_code' | 'codex';

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
}

export interface DiagnosticFile {
  label: string;
  path: string;
  exists: boolean;
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

export const BLINK_CONFIG = {
  WORKING: { frequency: 1, dutyCycle: 0.5 },
  NEEDS_INPUT: { frequency: 2, dutyCycle: 0.3 },
} as const;
