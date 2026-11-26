// Tema yalnızca <html data-theme> özniteliğini değiştirir; hiçbir uygulama verisine dokunmaz.

export const THEMES = ['system', 'dark', 'light'];

export function resolveTheme(pref, prefersDark) {
  if (pref === 'dark' || pref === 'light') return pref;
  return prefersDark ? 'dark' : 'light';
}

function systemPrefersDark() {
  return typeof window !== 'undefined' && window.matchMedia
    ? window.matchMedia('(prefers-color-scheme: dark)').matches
    : true;
}

export function applyTheme(pref) {
  const theme = resolveTheme(pref, systemPrefersDark());
  document.documentElement.dataset.theme = theme;
  return theme;
}

/** "system" tercihinde Windows renk modu değişince temayı günceller. Temizleme işlevi döndürür. */
export function watchSystemTheme(pref) {
  if (pref !== 'system' || !window.matchMedia) return () => {};
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const onChange = () => applyTheme('system');
  mq.addEventListener('change', onChange);
  return () => mq.removeEventListener('change', onChange);
}
