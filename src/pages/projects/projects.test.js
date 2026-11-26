import { describe, expect, it } from 'vitest';
import { parseArgs } from './ProjectEditor.jsx';

describe('parseArgs', () => {
  it('satır başına bir argüman; kabuk ayrıştırması yapmaz', () => {
    expect(parseArgs('.\n--flag value\n\n  x  ')).toEqual(['.', '--flag value', 'x']);
    expect(parseArgs('a && rm -rf /')).toEqual(['a && rm -rf /']);
  });
});
