// Değişken listesi filtreleme ve sıralama (saf işlevler, test edilebilir).

export const SCOPE_ORDER = { process: 0, user: 1, machine: 2 };

/**
 * @param {Array} vars  arka uçtan gelen VarView listesi
 * @param {object} f    { search, scope, category, onlySensitive, onlyIssues, sort }
 */
export function filterVars(vars, f) {
  const q = (f.search || '').trim().toLowerCase();
  let out = vars.filter((v) => {
    if (f.scope && f.scope !== 'all' && v.scope !== f.scope) return false;
    if (f.category && f.category !== 'all' && v.category !== f.category) return false;
    if (f.onlySensitive && !v.sensitive) return false;
    if (f.onlyIssues && !(v.issues && v.issues.length)) return false;
    if (q) {
      // Maskelenmiş değerlerde değer araması yapılamaz (değer ön uçta yoktur).
      const inName = v.name.toLowerCase().includes(q);
      const inValue = v.value != null && v.value.toLowerCase().includes(q);
      if (!inName && !inValue) return false;
    }
    return true;
  });
  const sort = f.sort || 'name';
  const byName = (a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
  const byScope = (a, b) => SCOPE_ORDER[a.scope] - SCOPE_ORDER[b.scope];
  out = [...out].sort((a, b) => {
    if (sort === 'scope') return byScope(a, b) || byName(a, b);
    if (sort === 'nameDesc') return -byName(a, b) || byScope(a, b);
    return byName(a, b) || byScope(a, b);
  });
  return out;
}

export function countBy(vars, key) {
  const out = {};
  for (const v of vars) out[v[key]] = (out[v[key]] || 0) + 1;
  return out;
}
