import { useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Modal } from '../../components/ui/Modal.jsx';
import { Button, Callout, Field } from '../../components/ui/primitives.jsx';
import VarRowsEditor, { rowsFromVars, rowsToInput, rowErrors } from '../../components/envset/VarRowsEditor.jsx';
import PathRowsEditor, { rowsFromPaths, pathRowsToInput, pathRowErrors } from '../../components/envset/PathRowsEditor.jsx';

export const CATEGORY_SUGGESTIONS = ['Node.js', 'Python', 'Rust', 'Java', 'Go', 'Web', '.NET', 'PHP', 'Docker', 'Veritabanı', 'Production', 'Kişisel', 'Oyun'];

export function parseTags(text) {
  return text
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
}

/** Profil oluşturma/düzenleme penceresi. `profile` yoksa yeni profil oluşturulur. */
export default function ProfileEditor({ profile, initial, onClose, onSaved }) {
  const t = useT();
  const toast = useToast();
  const base = profile || initial || {};
  const [name, setName] = useState(base.name || '');
  const [description, setDescription] = useState(base.description || '');
  const [category, setCategory] = useState(base.category || '');
  const [tags, setTags] = useState((base.tags || []).join(', '));
  const [vars, setVars] = useState(() => rowsFromVars(base.vars));
  const [paths, setPaths] = useState(() => rowsFromPaths(base.paths));
  const [note, setNote] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(null);
  const [showErrors, setShowErrors] = useState(false);

  const invalid = !name.trim() || rowErrors(vars).some(Boolean) || pathRowErrors(paths).some(Boolean);

  const save = async () => {
    setShowErrors(true);
    if (invalid) return;
    setBusy(true);
    setError(null);
    const input = {
      name: name.trim(),
      description,
      category,
      tags: parseTags(tags),
      metadata: profile ? profile.metadata : base.metadata || null,
      vars: rowsToInput(vars),
      paths: pathRowsToInput(paths),
    };
    try {
      if (profile) {
        await api.profileUpdate(profile.id, input, note.trim() || null);
        onSaved(profile.id);
      } else {
        onSaved(await api.profileCreate(input));
      }
      toast.ok(t('profiles.saved'));
    } catch (e) {
      setError(e);
      setBusy(false);
    }
  };

  return (
    <Modal
      xwide
      title={profile ? t('profiles.editTitle', { name: profile.name }) : t('profiles.newTitle')}
      onClose={busy ? undefined : onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" onClick={save} disabled={busy || (showErrors && invalid)}>
            {t('common.save')}
          </Button>
        </>
      }
    >
      <div className="stack">
        {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
        <div className="grid cols-2">
          <Field label={t('common.name')} error={showErrors && !name.trim() ? t('errors.invalid_profile_name') : null}>
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} aria-invalid={showErrors && !name.trim()} />
          </Field>
          <Field label={t('profiles.category')}>
            <input className="input" value={category} onChange={(e) => setCategory(e.target.value)} list="profile-categories" maxLength={60} />
            <datalist id="profile-categories">
              {CATEGORY_SUGGESTIONS.map((c) => (
                <option key={c} value={c} />
              ))}
            </datalist>
          </Field>
        </div>
        <Field label={t('common.description')}>
          <textarea className="textarea" rows={2} value={description} onChange={(e) => setDescription(e.target.value)} maxLength={2000} />
        </Field>
        <Field label={t('common.tags')} hint={t('profiles.tagsHint')}>
          <input className="input" value={tags} onChange={(e) => setTags(e.target.value)} />
        </Field>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('profiles.variables')}
          </div>
          <VarRowsEditor rows={vars} onChange={setVars} showErrors={showErrors} />
          <p className="field-hint" style={{ marginTop: 6 }}>
            {t('profiles.scopeHint')}
          </p>
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('profiles.pathChanges')}
          </div>
          <PathRowsEditor rows={paths} onChange={setPaths} showErrors={showErrors} />
        </div>
        {profile && (
          <Field label={t('profiles.changeNote')} hint={t('profiles.changeNoteHint')}>
            <input className="input" value={note} onChange={(e) => setNote(e.target.value)} maxLength={200} />
          </Field>
        )}
      </div>
    </Modal>
  );
}
