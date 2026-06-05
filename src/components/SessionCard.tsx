import type { Session } from '../lib/state-constants';
import { SOURCE_LABELS, STATE_LABELS } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface SessionCardProps {
  session: Session;
  expanded?: boolean;
  onToggle?: () => void;
}

export function SessionCard({ session, expanded = false, onToggle }: SessionCardProps) {
  const stateClass = `session-card--${session.state.toLowerCase().replace('_', '-')}`;
  const detail = session.detail || SOURCE_LABELS[session.source];

  return (
    <div
      className={`session-card ${stateClass}`}
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
      <div className="session-card__light">
        <TrafficLight state={session.state} compact />
      </div>
      <div className="session-card__info">
        <div className="session-card__topline">
          <div className="session-card__label">{session.label}</div>
          <span className="session-card__source">{SOURCE_LABELS[session.source]}</span>
        </div>
        <div className="session-card__detail">{expanded ? detail : SOURCE_LABELS[session.source]}</div>
      </div>
      <span className="session-card__state">
        {STATE_LABELS[session.state]}
      </span>
    </div>
  );
}
