import type { DisplayState } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface EmptyStateProps {
  state?: DisplayState;
}

export function EmptyState({ state = 'UNKNOWN' }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <TrafficLight state={state} size={24} showLabel={false} />
      <div className="empty-state__text">
        {state === 'UNKNOWN'
          ? '\u672a\u8fde\u63a5\u5230\u72b6\u6001\u5f15\u64ce'
          : '\u6ca1\u6709\u6d3b\u8dc3\u4f1a\u8bdd'}
      </div>
    </div>
  );
}
