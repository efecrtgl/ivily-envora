import { useT } from '../../i18n/index.jsx';
import { Button } from '../ui/primitives.jsx';
import { newKey } from './VarRowsEditor.jsx';

export function rowsFromPaths(paths = []) {
  return paths.map((p) => ({ key: newKey(), entry: p.entry, action: p.action, scope: p.scope }));
}

export function pathRowsToInput(rows) {
  return rows.map((r) => ({ entry: r.entry.trim(), action: r.action, scope: r.scope }));
}

export function pathRowErrors(rows) {
  const seen = new Set();
  return rows.map((r) => {
    const e = r.entry.trim();
    if (!e) return 'invalid_path_entry';
    if (e.includes(';')) return 'invalid_path_entry';
    const k = `${r.scope}|${e.toLowerCase().replace(/\//g, '\\').replace(/\\+$/, '')}`;
    if (seen.has(k)) return 'duplicate_path_entry';
    seen.add(k);
    return null;
  });
}

/** PATH değişiklikleri düzenleyici (başa ekle / sona ekle / kaldır), sıralanabilir. */
export default function PathRowsEditor({ rows, onChange, scopes = ['user', 'machine', 'process'], showErrors }) {
  const t = useT();
  const errors = pathRowErrors(rows);
  const update = (key, patch) => onChange(rows.map((r) => (r.key === key ? { ...r, ...patch } : r)));
  const move = (i, d) => {
    const j = i + d;
    if (j < 0 || j >= rows.length) return;
    const next = [...rows];
    [next[i], next[j]] = [next[j], next[i]];
    onChange(next);
  };
  return (
    <div className="stack tight">
      {rows.length > 0 && (
        <div className="table-wrap" style={{ border: '1px solid var(--border)', borderRadius: 10 }}>
          <table className="table fixed">
            <colgroup>
              <col />
              <col style={{ width: 150 }} />
              <col style={{ width: 118 }} />
              <col style={{ width: 110 }} />
            </colgroup>
            <thead>
              <tr>
                <th>{t('envset.pathEntry')}</th>
                <th>{t('envset.action')}</th>
                <th>{t('common.scope')}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.map((r, i) => (
                <tr key={r.key}>
                  <td>
                    <input
                      className="input mono"
                      value={r.entry}
                      onChange={(e) => update(r.key, { entry: e.target.value })}
                      placeholder="C:\\tools\\bin"
                      aria-invalid={showErrors && !!errors[i]}
                      title={showErrors && errors[i] ? t(`errors.${errors[i]}`) : undefined}
                      spellCheck={false}
                    />
                  </td>
                  <td>
                    <select className="select" value={r.action} onChange={(e) => update(r.key, { action: e.target.value })}>
                      <option value="prepend">{t('envset.prepend')}</option>
                      <option value="append">{t('envset.append')}</option>
                      <option value="remove">{t('envset.remove')}</option>
                    </select>
                  </td>
                  <td>
                    <select className="select" value={r.scope} onChange={(e) => update(r.key, { scope: e.target.value })}>
                      {scopes.map((s) => (
                        <option key={s} value={s}>
                          {t(`scope.${s}`)}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td className="actions">
                    <Button size="sm" variant="ghost" iconOnly icon="arrowUp" title={t('envset.moveUp')} onClick={() => move(i, -1)} disabled={i === 0} />
                    <Button size="sm" variant="ghost" iconOnly icon="arrowDown" title={t('envset.moveDown')} onClick={() => move(i, 1)} disabled={i === rows.length - 1} />
                    <Button size="sm" variant="ghost" iconOnly icon="x" title={t('common.delete')} onClick={() => onChange(rows.filter((x) => x.key !== r.key))} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <div className="row">
        <Button size="sm" icon="plus" onClick={() => onChange([...rows, { key: newKey(), entry: '', action: 'prepend', scope: scopes[0] }])}>
          {t('envset.addPath')}
        </Button>
        {showErrors && errors.some(Boolean) && <span className="field-error">{t('envset.fixErrors')}</span>}
      </div>
    </div>
  );
}
