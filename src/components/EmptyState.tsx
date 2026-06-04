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
        {state === 'UNKNOWN' ? '未连接到状态引擎' : '没有活跃会话'}
      </div>
    </div>
  );
}