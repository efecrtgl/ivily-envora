import { describe, expect, it } from 'vitest';
import { filterVars, countBy } from './envFilter.js';

const vars = [
  { name: 'Path', scope: 'machine', value: 'C:\\Windows', category: 'path', sensitive: false, issues: [] },
  { name: 'Path', scope: 'user', value: 'C:\\bin', category: 'path', sensitive: false, issues: [] },
  { name: 'GITHUB_TOKEN', scope: 'user', value: null, masked: true, category: 'credentials', sensitive: true, issues: [] },
  { name: 'java_home', scope: 'user', value: 'C:\\jdk ', category: 'development', sensitive: false, issues: ['valueWhitespace'] },
  { name: 'OS', scope: 'process', value: 'Windows_NT', category: 'system', sensitive: false, issues: [] },
];

describe('filterVars', () => {
  it('ada göre büyük/küçük harf duyarsız sıralar, eşitlikte kapsama göre', () => {
    const r = filterVars(vars, {});
    expect(r.map((v) => `${v.name}:${v.scope}`)).toEqual([
      'GITHUB_TOKEN:user',
      'java_home:user',
      'OS:process',
      'Path:user',
      'Path:machine',
    ]);
  });

  it('kapsam, kategori ve bayraklara göre filtreler', () => {
    expect(filterVars(vars, { scope: 'user' })).toHaveLength(3);
    expect(filterVars(vars, { category: 'path' })).toHaveLength(2);
    expect(filterVars(vars, { onlySensitive: true }).map((v) => v.name)).toEqual(['GITHUB_TOKEN']);
    expect(filterVars(vars, { onlyIssues: true }).map((v) => v.name)).toEqual(['java_home']);
  });

  it('adda ve maskelenmemiş değerde arar', () => {
    expect(filterVars(vars, { search: 'windows' }).map((v) => v.name)).toEqual(['OS', 'Path']);
    expect(filterVars(vars, { search: 'token' })).toHaveLength(1);
  });

  it('kapsam sıralaması', () => {
    expect(filterVars(vars, { sort: 'scope' })[0].scope).toBe('process');
  });

  it('countBy', () => {
    expect(countBy(vars, 'scope')).toEqual({ machine: 1, user: 3, process: 1 });
  });
});
