import { useState } from 'react';
import { useSessions } from './hooks/useSessions';
import { useDiagnostics } from './hooks/useDiagnostics';
import { useTheme } from './hooks/useTheme';
import { PopupLayout } from './components/PopupLayout';
import { Header } from './components/Header';
import { SessionList } from './components/SessionList';
import { SingleSessionView } from './components/SingleSessionView';
import { EmptyState } from './components/EmptyState';
import { AccessibilityLiveRegion } from './components/AccessibilityLiveRegion';
import { DiagnosticsPanel } from './components/DiagnosticsPanel';
import type { DisplayState, Session, SessionState } from './lib/state-constants';

function sessionPriority(state: SessionState) {
  switch (state) {
    case 'ERROR':
      return 4;
    case 'NEEDS_INPUT':
      return 3;
    case 'WORKING':
      return 2;
    case 'IDLE':
    default:
      return 1;
  }
}

function sortedSessionEntries(sessions: Record<string, Session>) {
  return Object.entries(sessions).sort(([, a], [, b]) => {
    const priority = sessionPriority(b.state) - sessionPriority(a.state);
    if (priority !== 0) return priority;
    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
  });
}

function App() {
  // Initialize theme
  useTheme();

  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const { sessions, recentEvents, aggregateState, loading } = useSessions();
  const diagnostics = useDiagnostics(diagnosticsOpen);

  const state = aggregateState as DisplayState;
  const entries = sortedSessionEntries(sessions);
  const sortedSessions = Object.fromEntries(entries);
  const count = entries.length;

  let content;
  if (loading) {
    content = (
      <div className="skeleton">
        <div className="skeleton__line" style={{ width: '60%' }} />
        <div className="skeleton__line" style={{ width: '40%' }} />
      </div>
    );
  } else if (count === 0) {
    content = <EmptyState state="IDLE" />;
  } else if (count === 1) {
    const firstEntry = entries[0];
    if (firstEntry) {
      const session = firstEntry[1];
      content = <SingleSessionView state={session.state} session={session} />;
    }
  } else {
    content = <SessionList sessions={sortedSessions} />;
  }

  return (
    <PopupLayout>
      <Header
        aggregateState={state}
        sessionCount={count}
        diagnosticsOpen={diagnosticsOpen}
        onToggleDiagnostics={() => setDiagnosticsOpen((value) => !value)}
      />
      {diagnosticsOpen && (
        <DiagnosticsPanel
          diagnostics={diagnostics.diagnostics}
          loading={diagnostics.loading}
          installing={diagnostics.installing}
          error={diagnostics.error}
          installMessage={diagnostics.installMessage}
          recentEvents={recentEvents}
          onRefresh={diagnostics.refresh}
          onInstallHooks={diagnostics.installHooks}
        />
      )}
      {content}
      <AccessibilityLiveRegion sessions={sessions} aggregateState={state} />
    </PopupLayout>
  );
}

export default App;
