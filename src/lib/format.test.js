import { describe, expect, it } from 'vitest';
import { formatBytes } from './format.js';
import { setIn, DEFAULT_SETTINGS } from '../app/settings.jsx';

describe('formatBytes', () => {
  it('birimleri seçer', () => {
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(2048)).toBe('2.0 KB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB');
    expect(formatBytes(-1)).toBe('—');
  });
});

describe('setIn', () => {
  it('iç içe alanı değiştirir ve diğer bölümlere dokunmaz', () => {
    const next = setIn(DEFAULT_SETTINGS, 'appearance.theme', 'dark');
    expect(next.appearance.theme).toBe('dark');
    expect(next.appearance.density).toBe(DEFAULT_SETTINGS.appearance.density);
    expect(next.security).toBe(DEFAULT_SETTINGS.security);
    expect(DEFAULT_SETTINGS.appearance.theme).toBe('system');
  });
});
