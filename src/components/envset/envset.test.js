import { describe, expect, it } from 'vitest';
import { rowsFromVars, rowsToInput, rowErrors } from './VarRowsEditor.jsx';
import { rowsFromPaths, pathRowsToInput, pathRowErrors } from './PathRowsEditor.jsx';

describe('değişken satırları', () => {
  it('maskeli değeri ön uca istemeden korur', () => {
    const rows = rowsFromVars([
      { name: 'TOKEN', value: null, sensitive: true, kind: null, scope: 'user', action: 'set' },
      { name: 'A', value: '1', sensitive: false, kind: 'string', scope: 'process', action: 'set' },
    ]);
    expect(rows[0].keep).toBe(true);
    const input = rowsToInput(rows);
    expect(input[0]).toMatchObject({ name: 'TOKEN', value: null, keep: true, previousName: 'TOKEN', sensitive: true });
    expect(input[1]).toMatchObject({ name: 'A', value: '1', keep: false });
  });

  it('kaldırma işleminde değer gönderilmez', () => {
    const rows = rowsFromVars([{ name: 'X', value: '', sensitive: false, kind: null, scope: 'user', action: 'unset' }]);
    expect(rowsToInput(rows)[0]).toMatchObject({ value: null, keep: false, action: 'unset' });
  });

  it('hataları bulur', () => {
    const rows = rowsFromVars([
      { name: 'A', value: '1', scope: 'user', action: 'set' },
      { name: 'a', value: '2', scope: 'user', action: 'set' },
      { name: 'B=C', value: '', scope: 'user', action: 'set' },
    ]);
    expect(rowErrors(rows)).toEqual([null, 'duplicate_var', 'invalid_name_equals']);
  });
});

describe('PATH satırları', () => {
  it('dönüştürür ve yinelenenleri bulur', () => {
    const rows = rowsFromPaths([
      { entry: 'C:\\a\\', action: 'prepend', scope: 'user' },
      { entry: 'c:/A', action: 'append', scope: 'user' },
      { entry: 'C:\\a', action: 'append', scope: 'machine' },
      { entry: 'x;y', action: 'append', scope: 'user' },
    ]);
    expect(pathRowErrors(rows)).toEqual([null, 'duplicate_path_entry', null, 'invalid_path_entry']);
    expect(pathRowsToInput(rows)[0]).toEqual({ entry: 'C:\\a\\', action: 'prepend', scope: 'user' });
  });
});
