import { useState } from 'react';
import type { Session } from '../lib/state-constants';
import { SessionCard } from './SessionCard';

interface SessionListProps {
  sessions: Record<string, Session>;
}

export function SessionList({ sessions }: SessionListProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const entries = Object.entries(sessions);

  return (
    <div className="session-list" role="list" aria-label="活跃 AI 会话">
      {entries.map(([id, session]) => (
        <SessionCard
          key={id}
          session={session}
          expanded={expandedId === id}
          onToggle={() => setExpandedId(expandedId === id ? null : id)}
        />
      ))}
    </div>
  );
}