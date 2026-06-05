import { useCallback, useEffect, useState } from 'react';
import type { Diagnostics, HookInstallResult } from '../lib/state-constants';

async function loadTauriInvoke() {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke;
}

export function useDiagnostics(enabled: boolean) {
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [installMessage, setInstallMessage] = useState<string | null>(null);

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

  const installHooks = useCallback(async () => {
    if (!enabled) return;

    setInstalling(true);
    setError(null);
    setInstallMessage(null);
    try {
      const invoke = await loadTauriInvoke();
      const result = await invoke<HookInstallResult>('install_hooks');
      setInstallMessage(
        result.ok
          ? `已修复 ${result.installed_count} 个 hooks`
          : 'hooks 写入完成，但仍有文件缺失'
      );
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(false);
    }
  }, [enabled, refresh]);

  return { diagnostics, loading, installing, error, installMessage, refresh, installHooks };
}
