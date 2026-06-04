import type { DisplayState, Session } from '../lib/state-constants';
import { STATE_LABELS } from '../lib/state-constants';

interface AccessibilityLiveRegionProps {
  sessions: Record<string, Session>;
  aggregateState: DisplayState;
}

export function AccessibilityLiveRegion({
  sessions,
  aggregateState,
}: AccessibilityLiveRegionProps) {
  const entries = Object.entries(sessions);
  const count = entries.length;

  let announcement: string;
  if (count === 0) {
    announcement = '没有活跃会话';
  } else if (count === 1) {
    const firstEntry = entries[0];
    if (firstEntry) {
      const session = firstEntry[1];
      announcement = `${session.label} 当前状态: ${STATE_LABELS[session.state as DisplayState]}${session.detail ? `, ${session.detail}` : ''}`;
    } else {
      announcement = '没有活跃会话';
    }
  } else {
    const urgentSessions = entries.filter(
      ([, s]) => s.state === 'NEEDS_INPUT' || s.state === 'ERROR'
    );
    if (urgentSessions.length > 0) {
      const firstUrgent = urgentSessions[0];
      const session = firstUrgent ? firstUrgent[1] : null;
      if (session) {
        announcement = `${session.label} 需要: ${STATE_LABELS[session.state === 'NEEDS_INPUT' ? 'NEEDS_INPUT' : 'ERROR']}`;
      } else {
        announcement = `${count} 个会话, 整体状态: ${STATE_LABELS[aggregateState]}`;
      }
    } else {
      announcement = `${count} 个会话, 整体状态: ${STATE_LABELS[aggregateState]}`;
    }
  }

  // NEEDS_INPUT and ERROR use assertive priority to interrupt screen readers
  const priority =
    aggregateState === 'NEEDS_INPUT' || aggregateState === 'ERROR'
      ? 'assertive'
      : 'polite';

  return (
    <div
      role="status"
      aria-live={priority}
      aria-atomic="true"
      style={{
        position: 'absolute',
        width: '1px',
        height: '1px',
        padding: '0',
        margin: '-1px',
        overflow: 'hidden',
        clip: 'rect(0, 0, 0, 0)',
        whiteSpace: 'nowrap',
        border: '0',
      }}
    >
      {announcement}
    </div>
  );
}