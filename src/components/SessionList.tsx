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
    <div className="session-list" role="list" aria-label="\u6d3b\u8dc3 AI \u4f1a\u8bdd">
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
