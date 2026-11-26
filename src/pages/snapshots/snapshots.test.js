import { describe, expect, it } from 'vitest';
import { filterSnapshots } from './Snapshots.jsx';

const list = [
  { name: 'Node öncesi', description: '', category: 'manual' },
  { name: 'user · Path', description: null, category: 'auto' },
];

describe('anlık görüntü filtresi', () => {
  it('türe ve metne göre süzer', () => {
    expect(filterSnapshots(list, { category: 'all', search: '' })).toHaveLength(2);
    expect(filterSnapshots(list, { category: 'auto', search: '' })[0].name).toBe('user · Path');
    expect(filterSnapshots(list, { category: 'all', search: 'node' })).toHaveLength(1);
  });
});
