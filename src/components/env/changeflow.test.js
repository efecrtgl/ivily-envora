import { describe, expect, it } from 'vitest';
import { pathEntryDiff } from './ChangeFlow.jsx';

describe('pathEntryDiff', () => {
  it('eklenen, kaldırılan ve sıra değişikliğini bulur', () => {
    expect(pathEntryDiff('C:\\a;C:\\b', 'C:\\n;C:\\a;C:\\b')).toEqual({ added: ['C:\\n'], removed: [], reordered: false });
    expect(pathEntryDiff('C:\\a;C:\\old', 'C:\\a')).toEqual({ added: [], removed: ['C:\\old'], reordered: false });
    expect(pathEntryDiff('C:\\a;C:\\b', 'C:\\b;C:\\a\\').reordered).toBe(true);
    expect(pathEntryDiff('C:/A/', 'c:\\a')).toEqual({ added: [], removed: [], reordered: false });
  });
});
