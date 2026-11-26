import { describe, expect, it } from 'vitest';
import { filterRows } from './Compare.jsx';
import { encodeSource, decodeSource } from './SourcePicker.jsx';

const rows = [
  { name: 'A', status: 'added', left: null, right: '1' },
  { name: 'B', status: 'unchanged', left: 'x', right: 'x' },
  { name: 'TOKEN', status: 'changed', left: null, right: null, sensitive: true },
];

describe('karşılaştırma filtreleri', () => {
  it('aynıları gizler ama "Aynı" sekmesinde gösterir', () => {
    expect(filterRows(rows, { hideUnchanged: true }).map((r) => r.name)).toEqual(['A', 'TOKEN']);
    expect(filterRows(rows, { status: 'unchanged', hideUnchanged: true }).map((r) => r.name)).toEqual(['B']);
  });
  it('ad ve değerde arar', () => {
    expect(filterRows(rows, { search: 'x' }).map((r) => r.name)).toEqual(['B']);
    expect(filterRows(rows, { search: 'tok' }).map((r) => r.name)).toEqual(['TOKEN']);
  });
});

describe('kaynak kodlama', () => {
  it('gidiş-dönüş', () => {
    for (const s of [{ kind: 'scope', scope: 'user' }, { kind: 'effective' }, { kind: 'profile', id: 3 }, { kind: 'snapshot', id: 12 }]) {
      expect(decodeSource(encodeSource(s))).toEqual(s);
    }
  });
});
