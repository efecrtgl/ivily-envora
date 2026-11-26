import { describe, expect, it } from 'vitest';
import { draftFromReport, draftValue, isDirty, addEntry, editEntry, removeIds, move, idsWithFlag, exists } from './pathDraft.js';

const report = {
  value: 'C:\\a;;C:\\a\\;Z:\\missing',
  entries: [
    { index: 0, raw: 'C:\\a', flags: [] },
    { index: 1, raw: '', flags: ['empty'] },
    { index: 2, raw: 'C:\\a\\', flags: ['duplicate'] },
    { index: 3, raw: 'Z:\\missing', flags: ['missing'] },
  ],
};

describe('PATH taslağı', () => {
  it('rapordan oluşturulan taslak aynı değeri verir (boş girdiler korunur)', () => {
    const d = draftFromReport(report);
    expect(draftValue(d)).toBe(report.value);
    expect(isDirty(d, report)).toBe(false);
  });

  it('bayrağa göre seçer ve yalnızca seçilenleri kaldırır', () => {
    const d = draftFromReport(report);
    const dup = idsWithFlag(d, 'duplicate');
    const missing = idsWithFlag(d, 'missing');
    const next = removeIds(d, [...dup, ...missing]);
    expect(draftValue(next)).toBe('C:\\a;');
    expect(isDirty(next, report)).toBe(true);
  });

  it('ekleme, düzenleme ve taşıma', () => {
    let d = draftFromReport({ value: 'C:\\x;C:\\y', entries: [{ raw: 'C:\\x', flags: [] }, { raw: 'C:\\y', flags: [] }] });
    d = addEntry(d, ' C:\\new ', 'start');
    expect(draftValue(d)).toBe('C:\\new;C:\\x;C:\\y');
    d = move(d, d[2].id, -1);
    expect(draftValue(d)).toBe('C:\\new;C:\\y;C:\\x');
    d = move(d, d[0].id, -1);
    expect(draftValue(d)).toBe('C:\\new;C:\\y;C:\\x');
    d = editEntry(d, d[0].id, 'D:\\tools');
    expect(d[0].edited).toBe(true);
    expect(exists(d, 'd:/TOOLS/')).toBe(true);
    expect(idsWithFlag(d, 'missing')).toEqual([]);
  });
});
