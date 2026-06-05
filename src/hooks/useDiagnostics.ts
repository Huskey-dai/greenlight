import { useCallback, useEffect, useState } from 'react';
import type { Diagnostics } from '../lib/state-constants';

async function loadTauriInvoke() {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke;
}

export function useDiagnostics(enabled: boolean) {
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!enabled) return;

    setLoading(true);
    setError(null);
    try {
      const invoke = await loadTauriInvoke();
      const result = await invoke<Diagnostics>('get_diagnostics');
      setDiagnostics(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  useEffect(() => {
    if (!enabled) return;
    void refresh();
  }, [enabled, refresh]);

  return { diagnostics, loading, error, refresh };
}
