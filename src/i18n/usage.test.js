import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import tr from './tr.js';

const SRC = join(dirname(fileURLToPath(import.meta.url)), '..');

function files(dir) {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) return name === 'i18n' ? [] : files(p);
    return /\.(jsx?|mjs)$/.test(name) && !name.includes('.test.') ? [p] : [];
  });
}

function has(dict, key) {
  return typeof key.split('.').reduce((o, k) => (o == null ? undefined : o[k]), dict) === 'string';
}

describe('i18n kullanımı', () => {
  it("kaynaktaki tüm sabit t('...') anahtarları sözlükte tanımlı", () => {
    const missing = [];
    for (const f of files(SRC)) {
      const text = readFileSync(f, 'utf8');
      for (const m of text.matchAll(/\bt\(\s*'([a-zA-Z0-9_.]+)'/g)) {
        if (!has(tr, m[1])) missing.push(`${f.slice(SRC.length + 1)}: ${m[1]}`);
      }
    }
    expect(missing).toEqual([]);
  });
});
