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

// State color constants (cross-theme)
export const STATE_COLORS: Record<DisplayState, string> = {
  IDLE: '#22C55E',
  WORKING: '#EAB308',
  NEEDS_INPUT: '#EF4444',
  ERROR: '#EF4444',
  UNKNOWN: '#6B7280',
} as const;

// State labels (Chinese, matching the requirements doc)
export const STATE_LABELS: Record<DisplayState, string> = {
  IDLE: '空闲',
  WORKING: '工作中',
  NEEDS_INPUT: '需授权',
  ERROR: '出错',
  UNKNOWN: '未连接',
} as const;

// Blink configuration
export const BLINK_CONFIG = {
  WORKING: { frequency: 1, dutyCycle: 0.5 },   // 1Hz, 50% duty
  NEEDS_INPUT: { frequency: 2, dutyCycle: 0.3 }, // 2Hz, 30% duty
} as const;