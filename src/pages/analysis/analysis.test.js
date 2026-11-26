import { describe, expect, it } from 'vitest';
import { filterFindings, ruleKey } from './Analysis.jsx';
import tr from '../../i18n/tr.js';

const RULE_IDS = [
  'path.missing', 'path.duplicate', 'path.crossScopeDuplicate', 'path.relative', 'path.empty', 'path.unresolved', 'path.length',
  'var.name', 'var.whitespace', 'var.unexpanded', 'var.undefinedReference', 'var.empty', 'var.shadowed', 'var.homeMissing',
  'tools.homeNotInPath', 'tools.storeAlias', 'system.tempMissing', 'system.pathext', 'security.machineSecret',
  'security.persistentSecret', 'process.stale', 'profile.conflict', 'profile.drift',
];

describe('analiz', () => {
  it('her kuralın başlığı ve açıklaması var', () => {
    for (const id of RULE_IDS) {
      const node = ruleKey(id).split('.').reduce((o, k) => o?.[k], tr);
      expect(node?.title, id).toBeTruthy();
      expect(node?.body, id).toBeTruthy();
    }
  });
  it('filtreler', () => {
    const f = [
      { rule: 'path.missing', category: 'path', severity: 'warning', target: 'C:\\x' },
      { rule: 'var.empty', category: 'variables', severity: 'info', target: 'FOO' },
    ];
    expect(filterFindings(f, { severity: 'warning', category: 'all', search: '' })).toHaveLength(1);
    expect(filterFindings(f, { severity: 'all', category: 'variables', search: '' })[0].target).toBe('FOO');
    expect(filterFindings(f, { severity: 'all', category: 'all', search: 'missing' })).toHaveLength(1);
  });
});
