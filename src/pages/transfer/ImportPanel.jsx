import { useMemo, useState } from 'react';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useNav } from '../../app/navigation.js';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Badge, Button, Callout, Card, Checkbox, Empty, Field, Segmented } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';

/** Varsayılan seçim: geçerli, değeri olan değişkenler. */
export function defaultSelection(vars) {
  return new Set(vars.filter((v) => !v.invalid && !v.excluded).map((v) => v.name));
}

function Warnings({ warnings }) {
  const t = useT();
  if (!warnings.length) return null;
  return (
    <Callout tone="warn" title={t('importer.warningsTitle', { count: warnings.length })}>
      <ul style={{ margin: '6px 0 0', paddingLeft: 18 }}>
        {warnings.slice(0, 20).map((w, i) => (
          <li key={i}>
            {w.line != null && <span className="mono">{t('importer.line', { line: w.line })}: </span>}
            {t(`importer.warn.${w.code}`)}
            {w.target && <span className="mono"> ({w.target})</span>}
          </li>
        ))}
      </ul>
    </Callout>
  );
}

function VariablesTarget({ preview, onDone }) {
  const t = useT();
  const toast = useToast();
  const { navigate } = useNav();
  const [selected, setSelected] = useState(() => defaultSelection(preview.vars));
  const [scope, setScope] = useState('user');
  const [policy, setPolicy] = useState('overwrite');
  const [flow, setFlow] = useState(null);
  const conflicts = preview.vars.filter((v) => selected.has(v.name) && v.existsIn.includes(scope)).length;

  const review = async () => {
    try {
      setFlow(await api.importVariablesPlan(preview.token, scope, [...selected], policy));
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };
  const toggle = (name) => {
    const s = new Set(selected);
    s.has(name) ? s.delete(name) : s.add(name);
    setSelected(s);
  };

  return (
    <>
      <div className="table-wrap" style={{ maxHeight: 360, border: '1px solid var(--border)', borderRadius: 10 }}>
        <table className="table fixed top">
          <colgroup>
            <col style={{ width: 40 }} />
            <col style={{ width: '30%' }} />
            <col />
            <col style={{ width: 170 }} />
          </colgroup>
          <thead>
            <tr>
              <th>
                <input
                  type="checkbox"
                  aria-label={t('common.all')}
                  checked={selected.size > 0 && selected.size === defaultSelection(preview.vars).size}
                  onChange={(e) => setSelected(e.target.checked ? defaultSelection(preview.vars) : new Set())}
                />
              </th>
              <th>{t('common.name')}</th>
              <th>{t('common.value')}</th>
              <th>{t('importer.status')}</th>
            </tr>
          </thead>
          <tbody>
            {preview.vars.map((v) => (
              <tr key={v.name}>
                <td>
                  <input type="checkbox" checked={selected.has(v.name)} disabled={!!v.invalid || v.excluded} onChange={() => toggle(v.name)} aria-label={v.name} />
                </td>
                <td className="name" title={v.name}>
                  {v.name}
                </td>
                <td className="small">
                  {v.excluded ? (
                    <span className="faint">{t('importer.excluded')}</span>
                  ) : v.sensitive ? (
                    <span className="mask faint">••••••••</span>
                  ) : v.action === 'unset' ? (
                    <Badge tone="danger">{t('envset.unset')}</Badge>
                  ) : (
                    <span className="mono break selectable">{v.value === '' ? t('vars.emptyValue') : v.value}</span>
                  )}
                </td>
                <td>
                  <div className="row">
                    {v.invalid && <Badge tone="danger">{t(`errors.${v.invalid}`)}</Badge>}
                    {v.sensitive && <Badge icon="lock">{t('vars.sensitive')}</Badge>}
                    {v.existsIn.map((s) => (
                      <ScopeBadge key={s} scope={s} />
                    ))}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="grid cols-2">
        <Field label={t('editor.targetScope')}>
          <Segmented
            label={t('editor.targetScope')}
            value={scope}
            onChange={setScope}
            options={[
              { value: 'user', label: t('scope.user') },
              { value: 'machine', label: t('scope.machine') },
            ]}
          />
        </Field>
        <Field label={t('importer.policy')} hint={t('importer.conflicts', { count: conflicts })}>
          <Segmented
            label={t('importer.policy')}
            value={policy}
            onChange={setPolicy}
            options={[
              { value: 'overwrite', label: t('importer.overwrite') },
              { value: 'skipExisting', label: t('importer.skipExisting') },
            ]}
          />
        </Field>
      </div>
      <div className="row">
        <span className="spacer" />
        <Button icon="compare" onClick={() => navigate('compare', { left: { kind: 'scope', scope }, right: { kind: 'import', token: preview.token } })}>
          {t('importer.compareFirst')}
        </Button>
        <Button variant="primary" icon="check" onClick={review} disabled={!selected.size}>
          {t('editor.review')}
        </Button>
      </div>
      {flow && <ChangeFlow plan={flow} title={t('importer.applyTitle', { scope: t(`scope.${scope}`) })} onClose={() => setFlow(null)} onDone={onDone} />}
    </>
  );
}

function PathTarget({ preview, onDone }) {
  const t = useT();
  const toast = useToast();
  const [scope, setScope] = useState('user');
  const [mode, setMode] = useState('appendMissing');
  const [flow, setFlow] = useState(null);
  return (
    <>
      <div className="list" style={{ border: '1px solid var(--border)', borderRadius: 10, maxHeight: 300, overflow: 'auto' }}>
        {preview.pathEntries.map((e, i) => (
          <div key={i} className="list-item mono small">
            {e}
          </div>
        ))}
      </div>
      <div className="grid cols-2">
        <Field label={t('editor.targetScope')}>
          <Segmented label={t('editor.targetScope')} value={scope} onChange={setScope} options={['user', 'machine'].map((s) => ({ value: s, label: t(`scope.${s}`) }))} />
        </Field>
        <Field label={t('importer.pathMode')} hint={t(`importer.pathModeHint.${mode}`)}>
          <Segmented
            label={t('importer.pathMode')}
            value={mode}
            onChange={setMode}
            options={['appendMissing', 'prependMissing', 'replace'].map((m) => ({ value: m, label: t(`importer.pathModes.${m}`) }))}
          />
        </Field>
      </div>
      <div className="row">
        <span className="spacer" />
        <Button
          variant="primary"
          icon="check"
          onClick={() => api.importPathPlan(preview.token, scope, mode).then(setFlow, (e) => toast.error(errorText(t, e)))}
        >
          {t('editor.review')}
        </Button>
      </div>
      {flow && <ChangeFlow plan={flow} title={t('pathmgr.applyTitle', { scope: t(`scope.${scope}`) })} onClose={() => setFlow(null)} onDone={onDone} />}
    </>
  );
}

function CreateTarget({ preview, onDone }) {
  const t = useT();
  const toast = useToast();
  const { navigate } = useNav();
  const suffix = t('importer.importedSuffix');
  const [name, setName] = useState(preview.nameTaken ? `${preview.name} ${suffix}` : preview.name || '');
  const [directory, setDirectory] = useState(preview.directory || '');
  const [keepLaunch, setKeepLaunch] = useState(false);
  const [busy, setBusy] = useState(false);
  const kind = preview.kind;

  const pickDir = async () => {
    const d = await openDialog({ directory: true, multiple: false });
    if (d) setDirectory(d);
  };

  const create = async () => {
    setBusy(true);
    try {
      const r = await api.importCreate({ token: preview.token, name: name.trim(), directory: kind === 'project' ? directory : null, keepLaunch });
      toast.ok(t('importer.created') + (r.skipped ? ` ${t('importer.skipped', { count: r.skipped })}` : ''));
      onDone();
      navigate({ profile: 'profiles', project: 'projects', snapshot: 'snapshots', template: 'templates' }[kind], kind === 'profile' || kind === 'project' ? { id: r.id } : null);
    } catch (e) {
      toast.error(errorText(t, e));
      setBusy(false);
    }
  };

  return (
    <div className="stack">
      <div className="row">
        <Badge>{t('profiles.varCount', { count: preview.vars.length || preview.snapshotItems })}</Badge>
        {preview.pathEntries.length > 0 && <Badge>{t('importer.pathCount', { count: preview.pathEntries.length })}</Badge>}
        {preview.profileName && <Badge icon="profile">{preview.profileName}</Badge>}
      </div>
      <Field label={t('common.name')} error={preview.nameTaken && name.trim().toLowerCase() === (preview.name || '').toLowerCase() ? t('errors.conflict') : null}>
        <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={120} />
      </Field>
      {kind === 'project' && (
        <>
          <Field label={t('projects.directory')} hint={!preview.directoryExists ? t('importer.dirMissing') : undefined}>
            <div className="row" style={{ flexWrap: 'nowrap' }}>
              <input className="input mono" value={directory} onChange={(e) => setDirectory(e.target.value)} spellCheck={false} />
              <Button icon="folder" onClick={pickDir}>
                {t('projects.browse')}
              </Button>
            </div>
          </Field>
          {preview.launchCustom && (
            <Callout tone="warn" title={t('importer.launchTitle')}>
              <div className="mono small break" style={{ margin: '6px 0' }}>
                {preview.launchCustom.program} {preview.launchCustom.args.join(' ')}
              </div>
              {t('importer.launchBody')}
              <div style={{ marginTop: 8 }}>
                <Checkbox checked={keepLaunch} onChange={setKeepLaunch}>
                  {t('importer.keepLaunch')}
                </Checkbox>
              </div>
            </Callout>
          )}
        </>
      )}
      <div className="row">
        <span className="spacer" />
        <Button variant="primary" icon="plus" onClick={create} disabled={busy || !name.trim() || (kind === 'project' && !directory.trim())}>
          {t(`importer.create.${kind}`)}
        </Button>
      </div>
    </div>
  );
}

export default function ImportPanel() {
  const t = useT();
  const toast = useToast();
  const [format, setFormat] = useState('auto');
  const [preview, setPreview] = useState(null);
  const [fileName, setFileName] = useState('');

  const pick = async () => {
    const path = await openDialog({
      multiple: false,
      filters: [
        { name: t('importer.supported'), extensions: ['json', 'csv', 'env', 'txt', 'local', 'development', 'production'] },
        { name: '*', extensions: ['*'] },
      ],
    });
    if (!path) return;
    try {
      if (preview) api.importDiscard(preview.token).catch(() => {});
      const p = await api.importPreview(path, format === 'auto' ? null : format);
      setPreview(p);
      setFileName(path.split(/[\\/]/).pop());
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const discard = () => {
    if (preview) api.importDiscard(preview.token).catch(() => {});
    setPreview(null);
  };

  const Target = useMemo(() => {
    if (!preview) return null;
    if (preview.kind === 'variables') return VariablesTarget;
    if (preview.kind === 'path') return PathTarget;
    return CreateTarget;
  }, [preview]);

  return (
    <Card title={t('importer.title')} icon="upload" desc={t('importer.desc')}>
      <div className="row">
        <Segmented
          label={t('transfer.format')}
          value={format}
          onChange={setFormat}
          options={[
            { value: 'auto', label: t('importer.auto') },
            { value: 'json', label: 'JSON' },
            { value: 'csv', label: 'CSV' },
            { value: 'env', label: '.env' },
            { value: 'text', label: t('transfer.textList') },
          ]}
        />
        <Button variant="primary" icon="folder" onClick={pick}>
          {t('importer.chooseFile')}
        </Button>
        {preview && (
          <Button variant="ghost" icon="x" onClick={discard}>
            {t('importer.discard')}
          </Button>
        )}
      </div>
      {!preview && (
        <Empty icon="upload" title={t('importer.emptyTitle')}>
          {t('importer.emptyBody')}
        </Empty>
      )}
      {preview && (
        <div className="stack" style={{ marginTop: 16 }}>
          <div className="row">
            <span className="mono small">{fileName}</span>
            <Badge>{preview.format.toUpperCase()}</Badge>
            <Badge tone="info">{t(`importer.kinds.${preview.kind}`)}</Badge>
          </div>
          {preview.containsSecrets && <Callout tone="warn" icon="lock">{t('importer.containsSecrets')}</Callout>}
          <Warnings warnings={preview.warnings} />
          <Target key={preview.token} preview={preview} onDone={discard} />
        </div>
      )}
    </Card>
  );
}
