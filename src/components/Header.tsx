import type { DisplayState } from '../lib/state-constants';
import { TrafficLight } from './TrafficLight';

interface HeaderProps {
  aggregateState: DisplayState;
  sessionCount: number;
}

export function Header({ aggregateState, sessionCount }: HeaderProps) {
  return (
    <div className="popup-header">
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <TrafficLight state={aggregateState} compact showLabel={false} />
        <span className="popup-header__title">Greenlight</span>
      </div>
      <span className="popup-header__count">
        {sessionCount === 0
          ? '无会话'
          : sessionCount === 1
            ? '1 个会话'
            : `${sessionCount} 个会话`}
      </span>
    </div>
  );
}