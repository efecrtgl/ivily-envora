import { useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Modal } from '../../components/ui/Modal.jsx';
import { Button, Callout, Field } from '../../components/ui/primitives.jsx';
import PathRowsEditor, { rowsFromPaths, pathRowsToInput, pathRowErrors } from '../../components/envset/PathRowsEditor.jsx';
import { newKey } from '../../components/envset/VarRowsEditor.jsx';
import { validateName } from '../../lib/envValidate.js';

/** Kullanıcı şablonu düzenleyici. Şablonlar gizli değer içeremez (arka uç da denetler). */
export default function TemplateEditor({ template, onClose, onSaved }) {
  const t = useT();
  const toast = useToast();
  const [name, setName] = useState(template?.name || '');
  const [description, setDescription] = useState(template?.description || '');
  const [category, setCategory] = useState(template?.category || '');
  const [vars, setVars] = useState(() => (template?.vars || []).map((v) => ({ ...v, key: newKey() })));
  const [paths, setPaths] = useState(() => rowsFromPaths(template?.paths));
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);

  const update = (key, patch) => setVars(vars.map((v) => (v.key === key ? { ...v, ...patch } : v)));
  const invalid = !name.trim() || vars.some((v) => validateName(v.name.trim())) || pathRowErrors(paths).some(Boolean);

  const save = async () => {
    setBusy(true);
    setError(null);
    const input = {
      name: name.trim(),
      description,
      category,
      vars: vars.map(({ key: _key, ...v }) => ({ ...v, name: v.name.trim() })),
      paths: pathRowsToInput(paths),
    };
    try {
      if (template && !template.builtin) await api.templateUpdate(template.id, input);
      else await api.templateCreate(input);
      toast.ok(t('templates.saved'));
      onSaved();
    } catch (e) {
      setError(e);
      setBusy(false);
    }
  };

  return (
    <Modal
      xwide
      title={template ? t('templates.editTitle', { name: template.name }) : t('templates.newTitle')}
      onClose={busy ? undefined : onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" onClick={save} disabled={busy || invalid}>
            {t('common.save')}
          </Button>
        </>
      }
    >
      <div className="stack">
        {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
        <Callout tone="info" icon="lock">
          {t('templates.noSecrets')}
        </Callout>
        <div className="grid cols-2">
          <Field label={t('common.name')}>
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} />
          </Field>
          <Field label={t('profiles.category')}>
            <input className="input" value={category} onChange={(e) => setCategory(e.target.value)} maxLength={60} />
          </Field>
        </div>
        <Field label={t('common.description')}>
          <textarea className="textarea" rows={2} value={description} onChange={(e) => setDescription(e.target.value)} maxLength={2000} />
        </Field>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('profiles.variables')}
          </div>
          {vars.length > 0 && (
            <div className="table-wrap" style={{ border: '1px solid var(--border)', borderRadius: 10 }}>
              <table className="table fixed">
                <colgroup>
                  <col style={{ width: '20%' }} />
                  <col style={{ width: '24%' }} />
                  <col style={{ width: 120 }} />
                  <col style={{ width: 130 }} />
                  <col />
                  <col style={{ width: 52 }} />
                </colgroup>
                <thead>
                  <tr>
                    <th>{t('common.name')}</th>
                    <th>{t('templates.exampleValue')}</th>
                    <th>{t('common.scope')}</th>
                    <th>{t('templates.roleCol')}</th>
                    <th>{t('templates.note')}</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {vars.map((v) => (
                    <tr key={v.key}>
                      <td>
                        <input className="input mono" value={v.name} onChange={(e) => update(v.key, { name: e.target.value })} aria-invalid={!!validateName(v.name.trim())} />
                      </td>
                      <td>
                        <input className="input mono" value={v.value} onChange={(e) => update(v.key, { value: e.target.value })} />
                      </td>
                      <td>
                        <select className="select" value={v.scope} onChange={(e) => update(v.key, { scope: e.target.value })}>
                          {['user', 'machine', 'process'].map((s) => (
                            <option key={s} value={s}>
                              {t(`scope.${s}`)}
                            </option>
                          ))}
                        </select>
                      </td>
                      <td>
                        <select className="select" value={v.role} onChange={(e) => update(v.key, { role: e.target.value })}>
                          {['required', 'recommended', 'optional'].map((r) => (
                            <option key={r} value={r}>
                              {t(`templates.role.${r}`)}
                            </option>
                          ))}
                        </select>
                      </td>
                      <td>
                        <input className="input" value={v.note} onChange={(e) => update(v.key, { note: e.target.value })} maxLength={300} />
                      </td>
                      <td className="actions">
                        <Button size="sm" variant="ghost" iconOnly icon="x" title={t('common.delete')} onClick={() => setVars(vars.filter((x) => x.key !== v.key))} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <Button size="sm" icon="plus" style={{ marginTop: 8 }} onClick={() => setVars([...vars, { key: newKey(), name: '', value: '', kind: null, scope: 'user', role: 'recommended', note: '' }])}>
            {t('envset.addVar')}
          </Button>
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('profiles.pathChanges')}
          </div>
          <PathRowsEditor rows={paths} onChange={setPaths} showErrors />
        </div>
      </div>
    </Modal>
  );
}
