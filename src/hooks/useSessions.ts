import { useState, useEffect } from 'react';
import type { SessionsPayload, Session, StateEvent } from '../lib/state-constants';

type DisplayState = 'IDLE' | 'WORKING' | 'NEEDS_INPUT' | 'ERROR' | 'UNKNOWN';

interface UseSessionsResult {
  sessions: Record<string, Session>;
  recentEvents: StateEvent[];
  aggregateState: DisplayState;
  loading: boolean;
  error: string | null;
}

async function loadTauriApi() {
  const { listen } = await import('@tauri-apps/api/event');
  const { invoke } = await import('@tauri-apps/api/core');
  return { listen, invoke };
}

export function useSessions(): UseSessionsResult {
  const [sessions, setSessions] = useState<Record<string, Session>>({});
  const [recentEvents, setRecentEvents] = useState<StateEvent[]>([]);
  const [aggregateState, setAggregateState] = useState<DisplayState>('UNKNOWN');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Initial fetch and event subscription
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let mounted = true;

    async function init() {
      try {
        const { listen, invoke } = await loadTauriApi();

        // Fetch initial state
        const result = await invoke<Record<string, Session>>('get_sessions');
        if (!mounted) return;
        setSessions(result);
        const agg = await invoke<string>('get_aggregate_state');
        if (!mounted) return;
        setAggregateState(agg as DisplayState);
        const events = await invoke<StateEvent[]>('get_recent_events');
        if (!mounted) return;
        setRecentEvents(events);
        setLoading(false);

        // Subscribe to updates
        const fn = await listen<SessionsPayload>('state-updated', (event) => {
          setSessions(event.payload.sessions);
          setAggregateState(event.payload.aggregate_state as DisplayState);
          setRecentEvents(event.payload.recent_events ?? []);
        });
        unlisten = fn;
      } catch (e) {
        if (mounted) {
          setError(String(e));
          setLoading(false);
        }
      }
    }

    init();

    return () => {
      mounted = false;
      if (unlisten) unlisten();
    };
  }, []);

  return { sessions, recentEvents, aggregateState, loading, error };
}
