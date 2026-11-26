import { useT } from '../../i18n/index.jsx';
import { Button, Badge } from '../ui/primitives.jsx';
import { validateName, looksSensitiveName } from '../../lib/envValidate.js';

let uid = 0;
export const newKey = () => `r${++uid}`;

/** Kayıtlı değişkenlerden düzenleyici satırları üretir. */
export function rowsFromVars(vars = []) {
  return vars.map((v) => ({
    key: newKey(),
    name: v.name,
    previousName: v.name,
    value: v.value ?? '',
    masked: v.value == null && v.action === 'set',
    keep: v.value == null && v.action === 'set',
    sensitive: v.sensitive ? true : null,
    kind: v.kind ?? null,
    scope: v.scope,
    action: v.action,
  }));
}

/** Düzenleyici satırlarını arka uç girdisine dönüştürür. */
export function rowsToInput(rows) {
  return rows.map((r) => ({
    name: r.name.trim(),
    value: r.action === 'unset' || r.keep ? null : r.value,
    keep: r.action === 'set' && r.keep,
    previousName: r.previousName || null,
    sensitive: r.sensitive,
    kind: r.kind,
    scope: r.scope,
    action: r.action,
  }));
}

export function rowErrors(rows) {
  const seen = new Set();
  return rows.map((r) => {
    const e = validateName(r.name.trim());
    if (e) return e;
    const k = r.name.trim().toUpperCase();
    if (seen.has(k)) return 'duplicate_var';
    seen.add(k);
    if (r.value.includes('\0')) return 'invalid_value_nul';
    return null;
  });
}

/**
 * Değişken listesi düzenleyici (profiller ve projeler için ortak).
 * `scopes`: seçilebilecek kapsamlar.
 */
export default function VarRowsEditor({ rows, onChange, scopes = ['user', 'machine', 'process'], showErrors }) {
  const t = useT();
  const errors = rowErrors(rows);
  const update = (key, patch) => onChange(rows.map((r) => (r.key === key ? { ...r, ...patch } : r)));
  const remove = (key) => onChange(rows.filter((r) => r.key !== key));
  const add = () =>
    onChange([
      ...rows,
      { key: newKey(), name: '', value: '', masked: false, keep: false, sensitive: null, kind: null, scope: scopes[0], action: 'set' },
    ]);

  return (
    <div className="stack tight">
      {rows.length > 0 && (
        <div className="table-wrap" style={{ border: '1px solid var(--border)', borderRadius: 10 }}>
          <table className="table fixed">
            <colgroup>
              <col style={{ width: '24%' }} />
              <col />
              <col style={{ width: 132 }} />
              <col style={{ width: 148 }} />
              <col style={{ width: 118 }} />
              <col style={{ width: 52 }} />
            </colgroup>
            <thead>
              <tr>
                <th>{t('common.name')}</th>
                <th>{t('common.value')}</th>
                <th>{t('common.scope')}</th>
                <th>{t('editor.kind')}</th>
                <th>{t('envset.action')}</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.map((r, i) => {
                const secretInput = r.sensitive === true || looksSensitiveName(r.name);
                return (
                  <tr key={r.key}>
                    <td>
                      <input
                        className="input mono"
                        value={r.name}
                        onChange={(e) => update(r.key, { name: e.target.value })}
                        aria-invalid={showErrors && !!errors[i]}
                        title={showErrors && errors[i] ? t(`errors.${errors[i]}`) : undefined}
                        spellCheck={false}
                        placeholder="NAME"
                      />
                    </td>
                    <td>
                      {r.action === 'unset' ? (
                        <span className="small faint">{t('envset.unsetHint')}</span>
                      ) : r.keep ? (
                        <div className="row" style={{ flexWrap: 'nowrap' }}>
                          <Badge tone="warn" icon="lock" title={t('envset.unchangedHint')}>
                            {t('envset.unchanged')}
                          </Badge>
                          <Button size="sm" variant="ghost" icon="edit" onClick={() => update(r.key, { keep: false, value: '' })}>
                            {t('envset.replace')}
                          </Button>
                        </div>
                      ) : (
                        <input
                          className="input mono"
                          type={secretInput ? 'password' : 'text'}
                          value={r.value}
                          onChange={(e) => update(r.key, { value: e.target.value })}
                          autoComplete="new-password"
                          spellCheck={false}
                        />
                      )}
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
                    <td>
                      <select className="select" value={r.kind ?? 'auto'} onChange={(e) => update(r.key, { kind: e.target.value === 'auto' ? null : e.target.value })} disabled={r.action === 'unset'}>
                        <option value="auto">{t('editor.kindAuto')}</option>
                        <option value="string">{t('kind.string')}</option>
                        <option value="expand">{t('kind.expand')}</option>
                      </select>
                    </td>
                    <td>
                      <select className="select" value={r.action} onChange={(e) => update(r.key, { action: e.target.value })}>
                        <option value="set">{t('envset.set')}</option>
                        <option value="unset">{t('envset.unset')}</option>
                      </select>
                    </td>
                    <td className="actions">
                      <Button size="sm" variant="ghost" iconOnly icon="x" title={t('common.delete')} onClick={() => remove(r.key)} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
      <div className="row">
        <Button size="sm" icon="plus" onClick={add}>
          {t('envset.addVar')}
        </Button>
        {showErrors && errors.some(Boolean) && <span className="field-error">{t('envset.fixErrors')}</span>}
      </div>
    </div>
  );
}
