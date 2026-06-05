import type { DisplayState } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface EmptyStateProps {
  state?: DisplayState;
}

export function EmptyState({ state = 'UNKNOWN' }: EmptyStateProps) {
  const title = state === 'UNKNOWN'
    ? '\u672a\u8fde\u63a5'
    : '\u7a7a\u95f2';
  const description = state === 'UNKNOWN'
    ? '\u72b6\u6001\u5f15\u64ce\u5c1a\u672a\u51c6\u5907\u5c31\u7eea'
    : '\u5f53\u524d\u6ca1\u6709\u6d3b\u8dc3\u4f1a\u8bdd';

  return (
    <div className="empty-state">
      <div className="empty-state__light">
        <TrafficLight state={state} size={64} showLabel={false} />
      </div>
      <div className="empty-state__title">{title}</div>
      <div className="empty-state__text">
        {description}
      </div>
    </div>
  );
}
