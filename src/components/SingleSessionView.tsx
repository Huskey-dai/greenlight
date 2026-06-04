import type { Session } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';
import { STATE_LABELS } from '../lib/state-constants';
import type { DisplayState } from '../lib/state-constants';

interface SingleSessionViewProps {
  state: DisplayState;
  session?: Session;
}

export function SingleSessionView({ state, session }: SingleSessionViewProps) {
  return (
    <div className="single-session">
      <TrafficLight state={state} size={48} />
      <div className="single-session__label">
        {session?.label ?? STATE_LABELS[state]}
      </div>
      {session?.detail && (
        <div className="single-session__detail">{session.detail}</div>
      )}
    </div>
  );
}