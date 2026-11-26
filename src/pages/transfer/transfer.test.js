import { describe, expect, it } from 'vitest';
import { defaultSelection } from './ImportPanel.jsx';
import { defaultFileName } from '../../components/transfer/ExportDialog.jsx';

describe('içe aktarma seçimi', () => {
  it('geçersiz ve değersizleri varsayılan olarak seçmez', () => {
    const s = defaultSelection([
      { name: 'A', invalid: null, excluded: false },
      { name: 'B=C', invalid: 'invalid_name_equals', excluded: false },
      { name: 'TOKEN', invalid: null, excluded: true },
    ]);
    expect([...s]).toEqual(['A']);
  });
});

describe('dışa aktarma dosya adı', () => {
  it('biçime göre uzantı', () => {
    expect(defaultFileName({ kind: 'variables' }, 'env')).toMatch(/^envora-degiskenler-\d{4}-\d{2}-\d{2}\.env$/);
    expect(defaultFileName({ kind: 'path', scope: 'user' }, 'text')).toMatch(/\.txt$/);
    expect(defaultFileName({ kind: 'profile', id: 3 }, 'json')).toMatch(/^envora-profile-3-/);
  });
});
