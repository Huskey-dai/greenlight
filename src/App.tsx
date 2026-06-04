import { useSessions } from './hooks/useSessions';
import { useTheme } from './hooks/useTheme';
import { PopupLayout } from './components/PopupLayout';
import { Header } from './components/Header';
import { SessionList } from './components/SessionList';
import { SingleSessionView } from './components/SingleSessionView';
import { EmptyState } from './components/EmptyState';
import { AccessibilityLiveRegion } from './components/AccessibilityLiveRegion';
import type { DisplayState } from './lib/state-constants';

function App() {
  // Initialize theme
  useTheme();

  const { sessions, aggregateState, loading } = useSessions();

  const state = aggregateState as DisplayState;
  const entries = Object.entries(sessions);
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
    content = <SessionList sessions={sessions} />;
  }

  return (
    <PopupLayout>
      <Header aggregateState={state} sessionCount={count} />
      {content}
      <AccessibilityLiveRegion sessions={sessions} aggregateState={state} />
    </PopupLayout>
  );
}

export default App;