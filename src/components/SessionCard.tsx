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
  const isDetectedProcess = session.id.startsWith('process-');
  const sourceHint =
    session.state === 'NEEDS_INPUT'
      ? '\u7b49\u5f85\u5904\u7406'
      : isDetectedProcess
        ? session.state === 'WORKING'
          ? '\u8fdb\u7a0b\u63a2\u6d4b\u5230\u6267\u884c'
          : '\u5ba2\u6237\u7aef\u5df2\u8fde\u63a5'
        : 'hook \u66f4\u65b0';
  const detail = session.detail || sourceHint;

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
        <div className="session-card__detail">{expanded ? detail : sourceHint}</div>
      </div>
      <span className="session-card__state">
        {STATE_LABELS[session.state]}
      </span>
    </div>
  );
}
