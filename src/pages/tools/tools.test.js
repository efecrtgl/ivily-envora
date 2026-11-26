import { describe, expect, it } from 'vitest';
import { javaHomeSuggestion, groupTools } from './Tools.jsx';
import { mergeVars, mergePaths, templateVars } from '../templates/Templates.jsx';

describe('araçlar', () => {
  it('JAVA_HOME önerisi bin dizininin üstüdür', () => {
    const tools = [{ id: 'java', found: true, appAlias: false, locations: [{ path: 'C:\\jdk-21\\bin\\java.exe' }] }];
    expect(javaHomeSuggestion(tools)).toBe('C:\\jdk-21');
    expect(javaHomeSuggestion([{ id: 'java', found: true, appAlias: false, locations: [{ path: 'C:\\x\\java.exe' }] }])).toBeNull();
    expect(javaHomeSuggestion([{ id: 'java', found: true, appAlias: true, locations: [{ path: 'C:\\jdk\\bin\\java.exe' }] }])).toBeNull();
  });
  it('kategorilere ayırır ve bulunamayanları gizler', () => {
    const g = groupTools([{ id: 'a', category: 'x', found: true }, { id: 'b', category: 'x', found: false }], false);
    expect(g.x.map((t) => t.id)).toEqual(['a']);
  });
});

describe('şablon birleştirme', () => {
  it('var olan değişkenlere dokunmaz', () => {
    const tpl = { vars: [{ name: 'NODE_ENV', value: 'development', scope: 'process' }, { name: 'PORT', value: '3000', scope: 'process' }] };
    const merged = mergeVars([{ name: 'node_env', value: 'production', scope: 'process', action: 'set' }], templateVars(tpl));
    expect(merged.map((v) => v.name)).toEqual(['node_env', 'PORT']);
    expect(merged[0].value).toBe('production');
    expect(mergePaths([{ entry: 'C:\\a', scope: 'user' }], [{ entry: 'c:\\A', scope: 'user' }, { entry: 'C:\\b', scope: 'user' }])).toHaveLength(2);
  });
});
