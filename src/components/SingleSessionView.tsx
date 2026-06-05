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
  const isDetectedProcess = session?.id.startsWith('process-') ?? false;
  const sourceHint = session
    ? state === 'NEEDS_INPUT'
      ? '\u7b49\u5f85\u4f60\u5904\u7406'
      : isDetectedProcess
        ? state === 'WORKING'
          ? '\u8fdb\u7a0b\u63a2\u6d4b\u5230\u6b63\u5728\u6267\u884c'
          : '\u5ba2\u6237\u7aef\u5df2\u8fde\u63a5'
        : '\u6765\u81ea hook \u66f4\u65b0'
    : null;
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
      {sourceHint && (
        <div className="single-session__source">{sourceHint}</div>
      )}
      {session?.detail && (
        <div className="single-session__detail">{session.detail}</div>
      )}
    </div>
  );
}
