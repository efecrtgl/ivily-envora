import { describe, expect, it } from 'vitest';
import { validateName, looksSensitiveName, hasPercentRef } from './envValidate.js';

describe('validateName', () => {
  it('arka uç kurallarıyla uyumlu', () => {
    expect(validateName('JAVA_HOME')).toBeNull();
    expect(validateName('ProgramFiles(x86)')).toBeNull();
    expect(validateName('')).toBe('invalid_name_empty');
    expect(validateName('   ')).toBe('invalid_name_empty');
    expect(validateName('A=B')).toBe('invalid_name_equals');
    expect(validateName('A\nB')).toBe('invalid_name_control');
    expect(validateName('X'.repeat(256))).toBe('invalid_name_length');
  });
});

describe('looksSensitiveName', () => {
  it('yaygın gizli adları tanır', () => {
    expect(looksSensitiveName('GITHUB_TOKEN')).toBe(true);
    expect(looksSensitiveName('openai_api_key')).toBe(true);
    expect(looksSensitiveName('DB_PASSWORD')).toBe(true);
    expect(looksSensitiveName('JAVA_HOME')).toBe(false);
    expect(looksSensitiveName('PATH')).toBe(false);
  });
});

describe('hasPercentRef', () => {
  it('%AD% başvurularını bulur', () => {
    expect(hasPercentRef('%USERPROFILE%\\bin')).toBe(true);
    expect(hasPercentRef('100% done')).toBe(false);
  });
});
