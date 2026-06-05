import type { Session } from '../lib/state-constants';
import { STATE_LABELS } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface SessionCardProps {
  session: Session;
  expanded?: boolean;
  onToggle?: () => void;
}

export function SessionCard({ session, expanded = false, onToggle }: SessionCardProps) {
  const isUrgent = session.state === 'NEEDS_INPUT' || session.state === 'ERROR';

  return (
    <div
      className={`session-card${isUrgent ? ' session-card--needs-input' : ''}`}
      onClick={onToggle}
      role="listitem"
      tabIndex={0}
      aria-expanded={expanded}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onToggle?.();
        }
      }}
      aria-label={`${session.label}: ${STATE_LABELS[session.state]}`}
    >
      <TrafficLight state={session.state} compact />
      <div className="session-card__info">
        <div className="session-card__label">{session.label}</div>
        {expanded && session.detail && (
          <div className="session-card__detail">{session.detail}</div>
        )}
      </div>
      <span className="session-card__state">
        {session.state}
      </span>
    </div>
  );
}
