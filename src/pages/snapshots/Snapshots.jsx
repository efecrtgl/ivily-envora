import { useMemo, useState } from 'react';
import { useNav } from '../../app/navigation.js';
import { useSettings } from '../../app/settings.jsx';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog, Modal } from '../../components/ui/Modal.jsx';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Checkbox, Empty, Field, Loading, PageHeader, SearchInput, Segmented, Toggle } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import SecretValue from '../../components/env/SecretValue.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';
import ExportDialog from '../../components/transfer/ExportDialog.jsx';

export function filterSnapshots(list, { category, search }) {
  const q = (search || '').trim().toLowerCase();
  return list.filter((s) => (category === 'all' || s.category === category) && (!q || s.name.toLowerCase().includes(q) || (s.description || '').toLowerCase().includes(q)));
}

function CreateDialog({ onClose, onCreated }) {
  const t = useT();
  const toast = useToast();
  const { settings } = useSettings();
  const lists = useAsync(() => Promise.all([api.profilesList(true), api.projectsList()]), []);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [kind, setKind] = useState('environment');
  const [scopes, setScopes] = useState({ user: true, machine: true, process: false });
  const [names, setNames] = useState('');
  const [sourceId, setSourceId] = useState('');
  const [exclude, setExclude] = useState(settings.security.excludeSensitiveInSnapshots);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(null);

  const selectedScopes = Object.entries(scopes).filter(([, v]) => v).map(([k]) => k);
  const valid = name.trim() && (kind !== 'environment' ? sourceId !== '' : selectedScopes.length > 0);

  const create = async () => {
    setBusy(true);
    setError(null);
    const nameList = names.split(/[\r\n,]+/).map((s) => s.trim()).filter(Boolean);
    const source =
      kind === 'environment'
        ? { kind, scopes: selectedScopes, names: nameList.length ? nameList : null }
        : { kind, id: Number(sourceId) };
    try {
      const r = await api.snapshotCreate({ name: name.trim(), description: description || null, source, excludeSensitive: exclude });
      toast.ok(t('snapshots.created', { count: r.items }) + (r.excludedSensitive ? ` ${t('snapshots.excludedCount', { count: r.excludedSensitive })}` : ''));
      onCreated(r.id);
    } catch (e) {
      setError(e);
      setBusy(false);
    }
  };

  const [profiles, projects] = lists.data || [[], []];
  return (
    <Modal
      title={t('snapshots.newTitle')}
      onClose={busy ? undefined : onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant="primary" onClick={create} disabled={busy || !valid}>
            {t('common.create')}
          </Button>
        </>
      }
    >
      <div className="stack">
        {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
        <Field label={t('common.name')}>
          <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={120} placeholder={t('snapshots.namePlaceholder')} />
        </Field>
        <Field label={t('common.description')}>
          <input className="input" value={description} onChange={(e) => setDescription(e.target.value)} maxLength={500} />
        </Field>
        <Field label={t('snapshots.source')}>
          <Segmented
            label={t('snapshots.source')}
            value={kind}
            onChange={(k) => {
              setKind(k);
              setSourceId('');
            }}
            options={['environment', 'profile', 'project'].map((k) => ({ value: k, label: t(`snapshots.sources.${k}`) }))}
          />
        </Field>
        {kind === 'environment' ? (
          <>
            <Field label={t('snapshots.scopes')}>
              <div className="row">
                {['user', 'machine', 'process'].map((s) => (
                  <Checkbox key={s} checked={scopes[s]} onChange={(v) => setScopes({ ...scopes, [s]: v })}>
                    {t(`scope.${s}`)}
                  </Checkbox>
                ))}
              </div>
            </Field>
            <Field label={t('snapshots.onlyNames')} hint={t('snapshots.onlyNamesHint')}>
              <textarea className="textarea mono" rows={3} value={names} onChange={(e) => setNames(e.target.value)} placeholder={'JAVA_HOME\nPath'} />
            </Field>
          </>
        ) : (
          <Field label={t(`snapshots.sources.${kind}`)} hint={kind === 'project' ? t('snapshots.projectHint') : undefined}>
            <select className="select" value={sourceId} onChange={(e) => setSourceId(e.target.value)}>
              <option value="">—</option>
              {(kind === 'profile' ? profiles : projects).map((x) => (
                <option key={x.id} value={x.id}>
                  {x.name}
                </option>
              ))}
            </select>
          </Field>
        )}
        <div className="row between">
          <div>
            <div className="setting-label">{t('snapshots.excludeSensitive')}</div>
            <div className="field-hint">{t('snapshots.excludeSensitiveHint')}</div>
          </div>
          <Toggle label={t('snapshots.excludeSensitive')} checked={exclude} onChange={setExclude} />
        </div>
      </div>
    </Modal>
  );
}

function SnapshotDetail({ id, onClose, onChanged }) {
  const t = useT();
  const lang = useLang();
  const toast = useToast();
  const { navigate } = useNav();
  const { data, error, reload } = useAsync(() => api.snapshotGet(id), [id]);
  const [selected, setSelected] = useState(new Set());
  const [flows, setFlows] = useState([]);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [renaming, setRenaming] = useState(null);
  const [exporting, setExporting] = useState(false);

  const restore = async () => {
    try {
      const names = selected.size ? [...selected] : null;
      const plans = await api.snapshotRestorePlans(id, names);
      setFlows(
        plans.map((plan, i) => ({
          plan,
          title: t('snapshots.restoreStep', { scope: t(`scope.${plan.scope}`), step: i + 1, total: plans.length }),
        })),
      );
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  if (error) return <Modal title={t('snapshots.title')} onClose={onClose}><Callout tone="danger">{errorText(t, error)}</Callout></Modal>;
  if (!data) return null;
  const { meta, items } = data;
  const restorable = items.some((i) => i.scope !== 'process');
  const toggle = (name) => {
    const s = new Set(selected);
    s.has(name) ? s.delete(name) : s.add(name);
    setSelected(s);
  };

  if (flows.length) {
    return (
      <ChangeFlow
        key={flows.length}
        plan={flows[0].plan}
        title={flows[0].title}
        context={{ kind: 'snapshotRestore', snapshotId: id }}
        onClose={() => setFlows((f) => f.slice(1))}
        onDone={onChanged}
      />
    );
  }

  return (
    <Modal
      xwide
      title={meta.name}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" icon="trash" onClick={() => setConfirmDelete(true)}>
            {t('common.delete')}
          </Button>
          <Button variant="ghost" icon="edit" onClick={() => setRenaming({ name: meta.name, description: meta.description || '' })}>
            {t('common.rename')}
          </Button>
          <Button variant="ghost" icon="download" onClick={() => setExporting(true)}>
            {t('common.export')}
          </Button>
          <span className="spacer" />
          <Button
            icon="compare"
            onClick={() => {
              const scope = meta.scopes.find((s) => s !== 'process') || 'user';
              onClose();
              navigate('compare', { left: { kind: 'snapshot', id }, right: meta.source === 'environment' ? { kind: 'scope', scope } : { kind: 'effective' } });
            }}
          >
            {t('snapshots.compareCurrent')}
          </Button>
          <Button variant="primary" icon="undo" onClick={restore} disabled={!restorable}>
            {selected.size ? t('snapshots.restoreSelected', { count: selected.size }) : t('snapshots.restoreAll')}
          </Button>
        </>
      }
    >
      <div className="stack">
        <div className="row">
          <Badge tone={meta.category === 'auto' ? 'info' : undefined}>{t(`snapshots.category.${meta.category}`)}</Badge>
          <Badge>{t(`snapshots.sources.${meta.source}`)}</Badge>
          {meta.scopes.map((s) => (
            <ScopeBadge key={s} scope={s} />
          ))}
          {meta.hasSecrets && (
            <Badge icon="lock" tone="warn">
              {t('snapshots.hasSecrets')}
            </Badge>
          )}
          <span className="spacer" />
          <span className="small muted">{formatDate(lang, meta.createdAt)}</span>
        </div>
        {meta.description && <p className="muted small">{meta.description}</p>}
        {!restorable && <Callout tone="info">{t('snapshots.processOnly')}</Callout>}
        <div className="table-wrap" style={{ maxHeight: 420, border: '1px solid var(--border)', borderRadius: 10 }}>
          <table className="table fixed top">
            <colgroup>
              <col style={{ width: 40 }} />
              <col style={{ width: '28%' }} />
              <col style={{ width: 118 }} />
              <col />
            </colgroup>
            <thead>
              <tr>
                <th />
                <th>{t('common.name')}</th>
                <th>{t('common.scope')}</th>
                <th>{t('common.value')}</th>
              </tr>
            </thead>
            <tbody>
              {items.map((i) => (
                <tr key={`${i.scope}:${i.name}`}>
                  <td>
                    {i.scope !== 'process' && <input type="checkbox" checked={selected.has(i.name)} onChange={() => toggle(i.name)} aria-label={i.name} />}
                  </td>
                  <td className="name" title={i.name}>
                    {i.name}
                  </td>
                  <td>
                    <ScopeBadge scope={i.scope} />
                  </td>
                  <td className="value small">
                    {!i.present ? (
                      <Badge tone="danger" title={t('snapshots.absentHint')}>
                        {t('snapshots.absent')}
                      </Badge>
                    ) : (
                      <SecretValue v={{ ...i, masked: i.sensitive }} multiline reveal={() => api.snapshotReveal(id, i.scope, i.name)} />
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="xsmall faint">{t('snapshots.restoreNote')}</p>
      </div>
      {exporting && <ExportDialog preset={{ kind: 'snapshot', id }} onClose={() => setExporting(false)} />}
      {confirmDelete && (
        <ConfirmDialog
          danger
          title={t('snapshots.deleteTitle', { name: meta.name })}
          confirmLabel={t('common.delete')}
          onCancel={() => setConfirmDelete(false)}
          onConfirm={async () => {
            try {
              await api.snapshotDelete(id);
              toast.ok(t('snapshots.deleted'));
              onChanged();
              onClose();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <p className="muted">{t('snapshots.deleteBody')}</p>
        </ConfirmDialog>
      )}
      {renaming && (
        <ConfirmDialog
          title={t('common.rename')}
          confirmLabel={t('common.save')}
          disabled={!renaming.name.trim()}
          onCancel={() => setRenaming(null)}
          onConfirm={async () => {
            try {
              await api.snapshotUpdate(id, renaming.name, renaming.description || null);
              setRenaming(null);
              reload();
              onChanged();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <div className="stack">
            <Field label={t('common.name')}>
              <input className="input" value={renaming.name} onChange={(e) => setRenaming({ ...renaming, name: e.target.value })} maxLength={120} />
            </Field>
            <Field label={t('common.description')}>
              <input className="input" value={renaming.description} onChange={(e) => setRenaming({ ...renaming, description: e.target.value })} maxLength={500} />
            </Field>
          </div>
        </ConfirmDialog>
      )}
    </Modal>
  );
}

export default function Snapshots() {
  const t = useT();
  const lang = useLang();
  const { loading, error, data, reload } = useAsync(() => api.snapshotsList(), []);
  const [category, setCategory] = useState('all');
  const [search, setSearch] = useState('');
  const [creating, setCreating] = useState(false);
  const [open, setOpen] = useState(null);
  const rows = useMemo(() => filterSnapshots(data || [], { category, search }), [data, category, search]);

  return (
    <div className="page">
      <PageHeader
        title={t('snapshots.title')}
        subtitle={t('snapshots.subtitle')}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
            {t('snapshots.new')}
          </Button>
        }
      />
      <div className="toolbar">
        <SearchInput value={search} onChange={setSearch} placeholder={t('common.search')} />
        <Segmented
          label={t('snapshots.categoryLabel')}
          value={category}
          onChange={setCategory}
          options={['all', 'manual', 'auto'].map((c) => ({ value: c, label: c === 'all' ? t('common.all') : t(`snapshots.category.${c}`) }))}
        />
      </div>
      {loading && !data && <Loading label={t('common.loading')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      {data && rows.length === 0 && (
        <Card>
          <Empty icon="snapshot" title={data.length ? t('vars.noResults') : t('snapshots.emptyTitle')}>
            {!data.length && t('snapshots.emptyBody')}
          </Empty>
        </Card>
      )}
      {rows.length > 0 && (
        <Card flush>
          <div className="list">
            {rows.map((s) => (
              <div key={s.id} className="list-item clickable" role="button" tabIndex={0} onClick={() => setOpen(s.id)} onKeyDown={(e) => e.key === 'Enter' && setOpen(s.id)}>
                <Icon name={s.category === 'auto' ? 'history' : 'snapshot'} size={18} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="row">
                    <strong className="truncate">{s.name}</strong>
                    {s.hasSecrets && <Icon name="lock" size={12} title={t('snapshots.hasSecrets')} />}
                  </div>
                  <div className="xsmall faint">
                    {formatDate(lang, s.createdAt)} · {t('profiles.varCount', { count: s.itemCount })}
                  </div>
                </div>
                <Badge tone={s.category === 'auto' ? 'info' : undefined}>{t(`snapshots.category.${s.category}`)}</Badge>
                <Badge>{t(`snapshots.sources.${s.source}`)}</Badge>
                {s.scopes.map((sc) => (
                  <ScopeBadge key={sc} scope={sc} />
                ))}
                <Icon name="chevronRight" size={14} />
              </div>
            ))}
          </div>
        </Card>
      )}
      {creating && (
        <CreateDialog
          onClose={() => setCreating(false)}
          onCreated={(id) => {
            setCreating(false);
            reload();
            setOpen(id);
          }}
        />
      )}
      {open && <SnapshotDetail id={open} onClose={() => setOpen(null)} onChanged={reload} />}
    </div>
  );
}
