import { createContext, useCallback, useContext, useMemo } from 'react';
import tr from './tr.js';
import en from './en.js';

export const DICTIONARIES = { tr, en };
export const LANGUAGES = [
  { value: 'tr', label: 'Türkçe' },
  { value: 'en', label: 'English' },
];
export const DEFAULT_LANG = 'tr';

function lookup(dict, key) {
  let cur = dict;
  for (const part of key.split('.')) {
    if (cur == null || typeof cur !== 'object') return undefined;
    cur = cur[part];
  }
  return typeof cur === 'string' ? cur : undefined;
}

/** Anahtarı çevirir. Bulunamazsa Türkçeye, o da yoksa anahtarın kendisine düşer. */
export function translate(lang, key, params) {
  let s = lookup(DICTIONARIES[lang] || tr, key) ?? lookup(tr, key) ?? key;
  if (params) {
    s = s.replace(/\{(\w+)\}/g, (m, k) => (params[k] != null ? String(params[k]) : m));
  }
  return s;
}

const I18nCtx = createContext({ lang: DEFAULT_LANG, t: (k, p) => translate(DEFAULT_LANG, k, p) });

export function I18nProvider({ lang, children }) {
  const l = DICTIONARIES[lang] ? lang : DEFAULT_LANG;
  const t = useCallback((key, params) => translate(l, key, params), [l]);
  const value = useMemo(() => ({ lang: l, t }), [l, t]);
  return <I18nCtx.Provider value={value}>{children}</I18nCtx.Provider>;
}

export function useT() {
  return useContext(I18nCtx).t;
}

export function useLang() {
  return useContext(I18nCtx).lang;
}

/** Arka uç hata nesnesini ({code, message}) kullanıcıya gösterilecek metne çevirir. */
export function errorText(t, err) {
  if (!err) return '';
  const code = typeof err === 'object' ? err.code : null;
  if (code) {
    const key = `errors.${code}`;
    const msg = t(key);
    if (msg !== key) return err.message && code.startsWith('invalid') ? `${msg} (${err.message})` : msg;
  }
  return typeof err === 'string' ? err : err.message || t('errors.internal');
}

export function formatDate(lang, iso) {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  return d.toLocaleString(lang === 'tr' ? 'tr-TR' : 'en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
