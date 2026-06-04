import { useState, useEffect } from 'react';

type Theme = 'light' | 'dark';

export function useTheme(): Theme {
  const [theme, setTheme] = useState<Theme>(() => {
    // Check OS preference first
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
    return 'light';
  });

  useEffect(() => {
    // Listen for OS theme changes
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => {
      setTheme(e.matches ? 'dark' : 'light');
    };
    mq.addEventListener('change', handler);

    // Also listen for Tauri theme-changed events
    let unlisten: (() => void) | null = null;
    async function subscribeTheme() {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        const fn = await listen<{ theme: string }>('theme-changed', (event) => {
          setTheme(event.payload.theme as Theme);
        });
        unlisten = fn;
      } catch {
        // Tauri not available (e.g., in dev mode without Tauri)
      }
    }
    subscribeTheme();

    return () => {
      mq.removeEventListener('change', handler);
      if (unlisten) unlisten();
    };
  }, []);

  // Apply theme to document
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  return theme;
}