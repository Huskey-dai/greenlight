import type { Diagnostics, SessionState, StateEvent } from '../lib/state-constants';

const TEXT = {
  ariaLabel: '\u8fd0\u884c\u8bca\u65ad',
  title: '\u8fd0\u884c\u8bca\u65ad',
  refresh: '\u5237\u65b0',
  loading: '\u8bfb\u53d6\u4e2d...',
  empty: '\u6682\u65e0\u8bca\u65ad\u6570\u636e',
  ok: '\u6b63\u5e38',
  missing: '\u7f3a\u5931',
  none: '\u65e0',
  version: '\u7248\u672c',
  httpPort: 'HTTP \u7aef\u53e3',
  notFound: '\u672a\u53d1\u73b0',
  sessions: '\u4f1a\u8bdd',
  sessionsUnit: '\u4e2a',
  detected: '\u63a2\u6d4b',
  processMonitor: '\u8fdb\u7a0b\u63a2\u6d4b',
  enabled: '\u5f00\u542f',
  disabled: '\u5173\u95ed',
  states: '\u72b6\u6001\u5206\u5e03',
  sources: '\u6765\u6e90\u5206\u5e03',
  latestUpdate: '\u6700\u540e\u66f4\u65b0',
  portFile: '\u7aef\u53e3\u6587\u4ef6',
  statusFile: '\u72b6\u6001\u6587\u4ef6',
  hooks: 'Hooks',
  repairHooks: '\u4fee\u590d hooks',
  repairing: '\u4fee\u590d\u4e2d...',
  hooksReady: '\u5df2\u5b89\u88c5',
  hooksMissing: '\u7f3a\u5931',
  recentEvents: '\u6700\u8fd1\u4e8b\u4ef6',
  noEvents: '\u6682\u65e0\u72b6\u6001\u53d8\u5316',
} as const;

interface DiagnosticsPanelProps {
  diagnostics: Diagnostics | null;
  loading: boolean;
  installing: boolean;
  error: string | null;
  installMessage: string | null;
  recentEvents: StateEvent[];
  onRefresh: () => void;
  onInstallHooks: () => void;
}

function yesNo(value: boolean) {
  return value ? TEXT.ok : TEXT.missing;
}

function formatCountMap(values: Record<string, number>) {
  const entries = Object.entries(values);
  if (entries.length === 0) return TEXT.none;
  return entries.map(([key, value]) => `${key}: ${value}`).join(' / ');
}

function formatEventTime(timestamp: string) {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return '--:--';
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function stateText(state?: SessionState) {
  if (!state) return '\u7ed3\u675f';
  const labels: Record<SessionState, string> = {
    IDLE: '\u7a7a\u95f2',
    WORKING: '\u5de5\u4f5c\u4e2d',
    NEEDS_INPUT: '\u9700\u6388\u6743',
    ERROR: '\u51fa\u9519',
  };
  return labels[state];
}

function sourceText(source: StateEvent['source']) {
  const labels: Record<StateEvent['source'], string> = {
    claude_code: 'Claude Code',
    codex: 'Codex \u5ba2\u6237\u7aef',
    detected_process: '\u8fdb\u7a0b\u63a2\u6d4b',
    system: '\u7cfb\u7edf',
  };
  return labels[source];
}

export function DiagnosticsPanel({
  diagnostics,
  loading,
  installing,
  error,
  installMessage,
  recentEvents,
  onRefresh,
  onInstallHooks,
}: DiagnosticsPanelProps) {
  return (
    <section className="diagnostics-panel" aria-label={TEXT.ariaLabel}>
      <div className="diagnostics-panel__header">
        <span className="diagnostics-panel__title">{TEXT.title}</span>
        <button
          className="diagnostics-panel__refresh"
          type="button"
          onClick={onRefresh}
          disabled={loading}
        >
          {TEXT.refresh}
        </button>
      </div>

      {error && <div className="diagnostics-panel__error">{error}</div>}
      {installMessage && <div className="diagnostics-panel__message">{installMessage}</div>}
      {!diagnostics && !error && (
        <div className="diagnostics-panel__empty">
          {loading ? TEXT.loading : TEXT.empty}
        </div>
      )}

      {diagnostics && (
        <div className="diagnostics-grid">
          <div className={`diagnostics-hooks-status${diagnostics.hooks_ready ? '' : ' diagnostics-hooks-status--missing'}`}>
            <div>
              <span>{TEXT.hooks}</span>
              <strong>{diagnostics.hooks_ready ? TEXT.hooksReady : TEXT.hooksMissing}</strong>
            </div>
            {!diagnostics.hooks_ready && (
              <button
                className="diagnostics-panel__repair"
                type="button"
                onClick={onInstallHooks}
                disabled={installing}
              >
                {installing ? TEXT.repairing : TEXT.repairHooks}
              </button>
            )}
          </div>
          <div className="diagnostics-row">
            <span>{TEXT.version}</span>
            <strong>{diagnostics.version}</strong>
          </div>
          <div className="diagnostics-row">
            <span>{TEXT.httpPort}</span>
            <strong>{diagnostics.http_port ?? TEXT.notFound}</strong>
          </div>
          <div className="diagnostics-row">
            <span>{TEXT.sessions}</span>
            <strong>
              {diagnostics.sessions_count} {TEXT.sessionsUnit} / {TEXT.detected}{' '}
              {diagnostics.process_sessions_count} {TEXT.sessionsUnit}
            </strong>
          </div>
          <div className="diagnostics-row">
            <span>{TEXT.processMonitor}</span>
            <strong>{diagnostics.process_monitor_enabled ? TEXT.enabled : TEXT.disabled}</strong>
          </div>
          <div className="diagnostics-row diagnostics-row--wide">
            <span>{TEXT.states}</span>
            <strong>{formatCountMap(diagnostics.by_state)}</strong>
          </div>
          <div className="diagnostics-row diagnostics-row--wide">
            <span>{TEXT.sources}</span>
            <strong>{formatCountMap(diagnostics.by_source)}</strong>
          </div>
          <div className="diagnostics-row diagnostics-row--wide">
            <span>{TEXT.latestUpdate}</span>
            <strong>{diagnostics.latest_update ?? TEXT.none}</strong>
          </div>
          <div className="diagnostics-row diagnostics-row--wide">
            <span>{TEXT.portFile}</span>
            <strong>{yesNo(diagnostics.port_file_exists)}</strong>
          </div>
          <div className="diagnostics-path">{diagnostics.port_file}</div>
          <div className="diagnostics-row diagnostics-row--wide">
            <span>{TEXT.statusFile}</span>
            <strong>{yesNo(diagnostics.status_file_exists)}</strong>
          </div>
          <div className="diagnostics-path">{diagnostics.status_file}</div>

          <div className="diagnostics-hooks">
            {diagnostics.hook_files.map((file) => (
              <div className="diagnostics-hook" key={file.path}>
                <span>{file.label}</span>
                <strong>{file.status === 'ok' ? TEXT.ok : TEXT.missing}</strong>
              </div>
            ))}
          </div>

          <div className="diagnostics-events">
            <div className="diagnostics-events__title">{TEXT.recentEvents}</div>
            {recentEvents.length === 0 ? (
              <div className="diagnostics-events__empty">{TEXT.noEvents}</div>
            ) : (
              recentEvents.slice().reverse().map((event) => (
                <div className="diagnostics-event" key={event.id}>
                  <span className="diagnostics-event__time">{formatEventTime(event.timestamp)}</span>
                  <span className="diagnostics-event__body">
                    {event.label} {stateText(event.from_state)} {'->'} {stateText(event.to_state)}
                    <em>{sourceText(event.source)}</em>
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </section>
  );
}
