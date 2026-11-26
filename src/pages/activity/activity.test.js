import { describe, expect, it } from 'vitest';
import { detailLines, CATEGORIES } from './Activity.jsx';
import { toggleSection, moveSection } from '../Settings.jsx';
import tr from '../../i18n/tr.js';

describe('etkinlik', () => {
  it('ayrıntıları satırlara çevirir', () => {
    expect(detailLines({ scope: 'user', count: 3, error: null, ctx: { a: 1 } })).toEqual([
      ['scope', 'user'],
      ['count', '3'],
      ['ctx', '{"a":1}'],
    ]);
    expect(detailLines(null)).toEqual([]);
  });
  it('her kategori çevrilmiş', () => {
    for (const c of CATEGORIES) expect(tr.activity.category[c], c).toBeTruthy();
  });
});

describe('pano bölümleri', () => {
  it('açma/kapama ve sıralama', () => {
    expect(toggleSection(['a', 'b'], 'c', true)).toEqual(['a', 'b', 'c']);
    expect(toggleSection(['a', 'b'], 'a', false)).toEqual(['b']);
    expect(moveSection(['a', 'b', 'c'], 'c', -1)).toEqual(['a', 'c', 'b']);
    expect(moveSection(['a', 'b'], 'a', -1)).toEqual(['a', 'b']);
  });
});
