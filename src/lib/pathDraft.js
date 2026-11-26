// PATH taslağı işlemleri (saf işlevler). Taslak, uygulanmadan önce kullanıcının yaptığı
// düzenlemeleri tutar; hiçbir şey otomatik kaldırılmaz veya sıralanmaz.

let seq = 0;
const nextId = () => `p${++seq}`;

export function draftFromReport(report) {
  return report.entries.map((e) => ({ id: nextId(), raw: e.raw, entry: e, edited: false }));
}

export function draftValue(draft) {
  return draft.map((d) => d.raw).join(';');
}

export function isDirty(draft, report) {
  return draftValue(draft) !== report.value;
}

export function keyOf(raw) {
  return raw.trim().replace(/^"|"$/g, '').replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
}

export function addEntry(draft, raw, position) {
  const item = { id: nextId(), raw: raw.trim(), entry: null, edited: true };
  return position === 'start' ? [item, ...draft] : [...draft, item];
}

export function editEntry(draft, id, raw) {
  return draft.map((d) => (d.id === id ? { ...d, raw, edited: true } : d));
}

export function removeIds(draft, ids) {
  const set = new Set(ids);
  return draft.filter((d) => !set.has(d.id));
}

export function move(draft, id, delta) {
  const i = draft.findIndex((d) => d.id === id);
  const j = i + delta;
  if (i < 0 || j < 0 || j >= draft.length) return draft;
  const next = [...draft];
  [next[i], next[j]] = [next[j], next[i]];
  return next;
}

/** Bayrağa göre öğe kimliklerini seçer (ör. 'duplicate', 'missing', 'empty'). */
export function idsWithFlag(draft, flag) {
  return draft.filter((d) => !d.edited && d.entry?.flags.includes(flag)).map((d) => d.id);
}

/** Taslakta yeni eklenen bir girdinin zaten var olup olmadığını denetler. */
export function exists(draft, raw) {
  const k = keyOf(raw);
  return draft.some((d) => keyOf(d.raw) === k);
}
