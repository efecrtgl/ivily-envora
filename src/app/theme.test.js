import { describe, expect, it } from 'vitest';
import { applyTheme, resolveTheme } from './theme.js';

describe('tema', () => {
  it('sistem tercihini çözer', () => {
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('dark', false)).toBe('dark');
  });

  it('yalnızca data-theme özniteliğini değiştirir, yerel depoya dokunmaz', () => {
    localStorage.setItem('envora.sentinel', 'kalsin');
    const before = { ...localStorage };
    applyTheme('light');
    expect(document.documentElement.dataset.theme).toBe('light');
    applyTheme('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
    expect({ ...localStorage }).toEqual(before);
  });
});
