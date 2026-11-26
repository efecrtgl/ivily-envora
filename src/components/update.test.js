import { describe, expect, it } from 'vitest';
import { shouldCheckOnStartup } from './UpdatePanel.jsx';

describe('açılış güncelleme denetimi', () => {
  const now = 10 * 24 * 3600 * 1000;
  it('ayar kapalıysa denetlemez', () => expect(shouldCheckOnStartup(false, 0, now)).toBe(false));
  it('hiç denetlenmediyse denetler', () => expect(shouldCheckOnStartup(true, null, now)).toBe(true));
  it('24 saatte en fazla bir kez', () => {
    expect(shouldCheckOnStartup(true, now - 3600 * 1000, now)).toBe(false);
    expect(shouldCheckOnStartup(true, now - 25 * 3600 * 1000, now)).toBe(true);
  });
});
