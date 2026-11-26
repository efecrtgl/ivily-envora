import { useState } from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Modal } from '../../components/ui/Modal.jsx';
import { Badge, Button, Callout, Field } from '../../components/ui/primitives.jsx';
import VarRowsEditor, { rowsFromVars, rowsToInput, rowErrors } from '../../components/envset/VarRowsEditor.jsx';
import PathRowsEditor, { rowsFromPaths, pathRowsToInput, pathRowErrors } from '../../components/envset/PathRowsEditor.jsx';
import { parseTags } from '../profiles/ProfileEditor.jsx';

const SCOPES = ['process'];

/** Argüman metnini satır başına bir argüman olacak şekilde ayırır (kabuk ayrıştırması yapılmaz). */
export function parseArgs(text) {
  return text
    .split(/\r?\n/)
    .map((s) => s.trim())
    .filter(Boolean);
}

export default function ProjectEditor({ project, onClose, onSaved }) {
  const t = useT();
  const toast = useToast();
  const profiles = useAsync(() => api.profilesList(false), []);
  const p = project || {};
  const [directory, setDirectory] = useState(p.directory || '');
  const [markers, setMarkers] = useState([]);
  const [name, setName] = useState(p.name || '');
  const [identifier, setIdentifier] = useState(p.identifier || '');
  const [description, setDescription] = useState(p.description || '');
  const [tags, setTags] = useState((p.tags || []).join(', '));
  const [profileId, setProfileId] = useState(p.profileId ?? '');
  const [shell, setShell] = useState(p.launch?.shell || '');
  const [program, setProgram] = useState(p.launch?.custom?.program || '');
  const [args, setArgs] = useState((p.launch?.custom?.args || []).join('\n'));
  const [vars, setVars] = useState(() => rowsFromVars(p.vars));
  const [paths, setPaths] = useState(() => rowsFromPaths(p.paths));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(null);
  const [showErrors, setShowErrors] = useState(false);

  const pickDir = async () => {
    const dir = await openDialog({ directory: true, multiple: false, defaultPath: directory || undefined });
    if (!dir) return;
    try {
      const info = await api.projectInspectDir(dir);
      setDirectory(info.path);
      setMarkers(info.markers);
      if (!name.trim()) setName(info.suggestedName);
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const invalid = !name.trim() || !directory.trim() || rowErrors(vars).some(Boolean) || pathRowErrors(paths).some(Boolean);

  const save = async () => {
    setShowErrors(true);
    if (invalid) return;
    setBusy(true);
    setError(null);
    const input = {
      name: name.trim(),
      identifier: identifier.trim() || null,
      description,
      directory,
      profileId: profileId === '' ? null : Number(profileId),
      tags: parseTags(tags),
      launch: { shell: shell || null, custom: program.trim() ? { program: program.trim(), args: parseArgs(args) } : null },
      vars: rowsToInput(vars),
      paths: pathRowsToInput(paths),
    };
    try {
      if (project) {
        await api.projectUpdate(project.id, input);
        onSaved(project.id);
      } else {
        onSaved(await api.projectCreate(input));
      }
      toast.ok(t('projects.saved'));
    } catch (e) {
      setError(e);
      setBusy(false);
    }
  };

  return (
    <Modal
      xwide
      title={project ? t('projects.editTitle', { name: project.name }) : t('projects.newTitle')}
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
        <Field label={t('projects.directory')} error={showErrors && !directory.trim() ? t('errors.invalid_directory') : null}>
          <div className="row" style={{ flexWrap: 'nowrap' }}>
            <input className="input mono" value={directory} onChange={(e) => setDirectory(e.target.value)} placeholder="C:\\src\\my-app" spellCheck={false} />
            <Button icon="folder" onClick={pickDir}>
              {t('projects.browse')}
            </Button>
          </div>
          {markers.length > 0 && (
            <div className="row" style={{ marginTop: 6 }}>
              <span className="xsmall faint">{t('projects.detected')}</span>
              {markers.map((m) => (
                <Badge key={m}>{t(`marker.${m}`)}</Badge>
              ))}
            </div>
          )}
        </Field>
        <div className="grid cols-2">
          <Field label={t('common.name')} error={showErrors && !name.trim() ? t('errors.invalid_project_name') : null}>
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} />
          </Field>
          <Field label={t('projects.identifier')} hint={t('projects.identifierHint')}>
            <input className="input mono" value={identifier} onChange={(e) => setIdentifier(e.target.value.toLowerCase())} maxLength={64} placeholder="my-app" />
          </Field>
        </div>
        <Field label={t('common.description')}>
          <textarea className="textarea" rows={2} value={description} onChange={(e) => setDescription(e.target.value)} maxLength={2000} />
        </Field>
        <div className="grid cols-2">
          <Field label={t('projects.profile')} hint={t('projects.profileHint')}>
            <select className="select" value={profileId} onChange={(e) => setProfileId(e.target.value)}>
              <option value="">{t('common.none')}</option>
              {(profiles.data || []).map((f) => (
                <option key={f.id} value={f.id}>
                  {f.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label={t('common.tags')} hint={t('profiles.tagsHint')}>
            <input className="input" value={tags} onChange={(e) => setTags(e.target.value)} />
          </Field>
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('projects.vars')}
          </div>
          <VarRowsEditor rows={vars} onChange={setVars} scopes={SCOPES} showErrors={showErrors} />
        </div>
        <div>
          <div className="field-label" style={{ marginBottom: 6 }}>
            {t('profiles.pathChanges')}
          </div>
          <PathRowsEditor rows={paths} onChange={setPaths} scopes={SCOPES} showErrors={showErrors} />
        </div>
        <div className="card" style={{ padding: 16 }}>
          <div className="card-title" style={{ marginBottom: 12 }}>
            {t('projects.launchConfig')}
          </div>
          <div className="grid cols-2">
            <Field label={t('projects.shell')}>
              <select className="select" value={shell} onChange={(e) => setShell(e.target.value)}>
                <option value="">{t('projects.shellDefault')}</option>
                <option value="pwsh">PowerShell 7 (pwsh)</option>
                <option value="powershell">Windows PowerShell</option>
                <option value="cmd">{t('projects.cmd')}</option>
              </select>
            </Field>
            <Field label={t('projects.customProgram')} hint={t('projects.customProgramHint')}>
              <input className="input mono" value={program} onChange={(e) => setProgram(e.target.value)} placeholder="code.exe" spellCheck={false} />
            </Field>
          </div>
          {program.trim() && (
            <Field label={t('projects.customArgs')} hint={t('projects.customArgsHint')}>
              <textarea className="textarea mono" rows={3} value={args} onChange={(e) => setArgs(e.target.value)} placeholder="." spellCheck={false} />
            </Field>
          )}
        </div>
      </div>
    </Modal>
  );
}
