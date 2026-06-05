import type { Session } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';
import { SOURCE_LABELS, STATE_LABELS, STATE_SUMMARIES } from '../lib/state-constants';
import type { DisplayState } from '../lib/state-constants';

interface SingleSessionViewProps {
  state: DisplayState;
  session?: Session;
}

export function SingleSessionView({ state, session }: SingleSessionViewProps) {
  const sourceLabel = session ? SOURCE_LABELS[session.source] : null;
  const title = session?.label ?? STATE_LABELS[state];
  const summary = session
    ? `${sourceLabel} ${STATE_SUMMARIES[state]}`
    : STATE_SUMMARIES[state];

  return (
    <div className="single-session">
      <div className="single-session__light">
        <TrafficLight state={state} size={82} showLabel={false} />
      </div>
      <div className="single-session__status">
        {STATE_LABELS[state]}
      </div>
      <div className="single-session__label">{title}</div>
      <div className="single-session__summary">{summary}</div>
      {session?.detail && (
        <div className="single-session__detail">{session.detail}</div>
      )}
    </div>
  );
}
