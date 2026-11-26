import { describe, expect, it } from 'vitest';
import tr from './tr.js';
import en from './en.js';
import { translate, loadLanguage } from './index.jsx';

function keys(obj, prefix = '') {
  return Object.entries(obj).flatMap(([k, v]) =>
    v && typeof v === 'object' ? keys(v, `${prefix}${k}.`) : [`${prefix}${k}`],
  );
}

describe('i18n', () => {
  it('Türkçe ve İngilizce sözlükler aynı anahtarlara sahip', () => {
    const a = keys(tr).sort();
    const b = keys(en).sort();
    expect(a.filter((k) => !b.includes(k))).toEqual([]);
    expect(b.filter((k) => !a.includes(k))).toEqual([]);
  });

  it('hiçbir çeviri boş değil', () => {
    for (const [name, dict] of Object.entries({ tr, en })) {
      for (const k of keys(dict)) {
        const v = k.split('.').reduce((o, p) => o[p], dict);
        expect(typeof v === 'string' && v.trim().length > 0, `${name}:${k}`).toBe(true);
      }
    }
  });

  it('İngilizce yüklenene kadar Türkçeye düşer, yüklendikten sonra çevirir', async () => {
    expect(translate('en', 'common.save')).toBe('Kaydet');
    await loadLanguage('en');
    expect(translate('en', 'common.save')).toBe('Save');
  });

  it('parametreleri yerleştirir ve bilinmeyenleri olduğu gibi bırakır', async () => {
    await loadLanguage('en');
    expect(translate('tr', 'common.save')).toBe('Kaydet');
    expect(translate('en', 'common.save')).toBe('Save');
    expect(translate('xx', 'common.save')).toBe('Kaydet');
    expect(translate('tr', 'missing.key')).toBe('missing.key');
  });
});
