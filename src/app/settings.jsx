import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { applyTheme, watchSystemTheme } from './theme.js';

// Görünüm tercihleri (tema, dil). Kalıcı depolama aşama 4'te arka uca taşınır;
// tercih edilen tema ilk çizimden önce uygulanabilsin diye yerel bir kopya da tutulur.
const LOCAL_KEY = 'envora.ui';

export const DEFAULT_UI = { theme: 'system', language: 'tr' };

function readLocal() {
  try {
    const raw = localStorage.getItem(LOCAL_KEY);
    return raw ? { ...DEFAULT_UI, ...JSON.parse(raw) } : DEFAULT_UI;
  } catch {
    return DEFAULT_UI;
  }
}

function writeLocal(ui) {
  try {
    localStorage.setItem(LOCAL_KEY, JSON.stringify({ theme: ui.theme, language: ui.language }));
  } catch {
    /* depolama kullanılamıyorsa yalnızca bu oturum için geçerli olur */
  }
}

const SettingsCtx = createContext(null);

export function SettingsProvider({ children }) {
  const [ui, setUi] = useState(readLocal);

  useEffect(() => {
    applyTheme(ui.theme);
    return watchSystemTheme(ui.theme);
  }, [ui.theme]);

  useEffect(() => {
    document.documentElement.lang = ui.language;
  }, [ui.language]);

  const update = useCallback((patch) => {
    setUi((prev) => {
      const next = { ...prev, ...patch };
      writeLocal(next);
      return next;
    });
  }, []);

  const value = useMemo(() => ({ ui, update }), [ui, update]);
  return <SettingsCtx.Provider value={value}>{children}</SettingsCtx.Provider>;
}

export function useSettings() {
  return useContext(SettingsCtx);
}
