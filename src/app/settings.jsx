import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { applyTheme, watchSystemTheme } from './theme.js';
import { api, inTauri } from '../lib/api.js';

// Ayarların asıl kaynağı arka uçtaki veritabanıdır (doğrulanmış).
// Tema ve dilin yerel bir kopyası yalnızca ilk çizimde doğru temayı göstermek için tutulur.
const LOCAL_KEY = 'envora.ui';

export const DEFAULT_SETTINGS = {
  general: { language: 'tr', startPage: 'dashboard' },
  appearance: { theme: 'system', density: 'comfortable', reduceMotion: false },
  dashboard: { sections: ['overview', 'pathHealth', 'analysis', 'security', 'profiles', 'projects', 'tools', 'activity'] },
  security: {
    autoHideSeconds: 30,
    warnOnCopy: true,
    clipboardClearSeconds: 30,
    excludeSensitiveOnExport: true,
    excludeSensitiveInSnapshots: false,
  },
  backup: { autoSnapshotBeforeChanges: true, maxAutoSnapshots: 50 },
  activity: { enabled: true, retentionDays: 90 },
  notifications: { showSuccess: true, restartHint: true },
  updates: { channel: 'stable', checkOnStartup: true },
  performance: { probeToolVersions: true, probeTimeoutMs: 3000 },
  workspace: { defaultShell: 'powershell' },
  ecosystem: { iepEnabled: false },
  analysis: { disabledRules: [], severity: {} },
};

function readLocal() {
  try {
    const raw = JSON.parse(localStorage.getItem(LOCAL_KEY) || '{}');
    return {
      ...DEFAULT_SETTINGS,
      general: { ...DEFAULT_SETTINGS.general, language: raw.language || 'tr' },
      appearance: { ...DEFAULT_SETTINGS.appearance, theme: raw.theme || 'system' },
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function writeLocal(s) {
  try {
    localStorage.setItem(LOCAL_KEY, JSON.stringify({ theme: s.appearance.theme, language: s.general.language }));
  } catch {
    /* depolama kullanılamıyorsa yalnızca bu oturum için geçerli olur */
  }
}

/** İç içe bir ayar alanını değiştirilemez biçimde günceller: set(s, 'appearance.theme', 'dark') */
export function setIn(obj, path, value) {
  const [head, ...rest] = path.split('.');
  return { ...obj, [head]: rest.length ? setIn(obj[head] || {}, rest.join('.'), value) : value };
}

const SettingsCtx = createContext(null);

export function SettingsProvider({ children }) {
  const [settings, setSettings] = useState(readLocal);
  const [meta, setMeta] = useState({ loaded: !inTauri(), fixed: [], degraded: null });
  const latest = useRef(settings);
  latest.current = settings;

  useEffect(() => {
    if (!inTauri()) return;
    api
      .settingsGet()
      .then((p) => {
        setSettings(p.settings);
        writeLocal(p.settings);
        setMeta({ loaded: true, fixed: p.fixed || [], degraded: p.degraded, restoreNote: p.restoreNote });
      })
      .catch(() => setMeta((m) => ({ ...m, loaded: true })));
  }, []);

  const { theme, density, reduceMotion } = settings.appearance;
  useEffect(() => {
    applyTheme(theme);
    return watchSystemTheme(theme);
  }, [theme]);

  useEffect(() => {
    const el = document.documentElement;
    el.dataset.density = density;
    el.dataset.reduceMotion = reduceMotion ? 'true' : 'false';
    el.lang = settings.general.language;
  }, [density, reduceMotion, settings.general.language]);

  /** Ayarları kaydeder. Arka uç reddederse önceki değerlere döner ve hatayı fırlatır. */
  const save = useCallback(async (next) => {
    const prev = latest.current;
    setSettings(next);
    writeLocal(next);
    if (!inTauri()) return next;
    try {
      const saved = await api.settingsUpdate(next);
      setSettings(saved);
      return saved;
    } catch (e) {
      setSettings(prev);
      writeLocal(prev);
      throw e;
    }
  }, []);

  const set = useCallback((path, value) => save(setIn(latest.current, path, value)), [save]);

  const reset = useCallback(async () => {
    const s = inTauri() ? await api.settingsReset() : DEFAULT_SETTINGS;
    setSettings(s);
    writeLocal(s);
    return s;
  }, []);

  const value = useMemo(() => ({ settings, meta, save, set, reset }), [settings, meta, save, set, reset]);
  return <SettingsCtx.Provider value={value}>{children}</SettingsCtx.Provider>;
}

export function useSettings() {
  return useContext(SettingsCtx);
}
