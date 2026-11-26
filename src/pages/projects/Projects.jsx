import { useState } from 'react';
import { useNav } from '../../app/navigation.js';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog } from '../../components/ui/Modal.jsx';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Empty, Loading, PageHeader, SearchInput, Tabs } from '../../components/ui/primitives.jsx';
import { VarsTable, PathsTable } from '../profiles/ProfileDetail.jsx';
import ProjectEditor from './ProjectEditor.jsx';
import LaunchDialog from './LaunchDialog.jsx';

const KIND_TONE = { added: 'ok', changed: 'warn', removed: 'danger' };

function EnvPreview({ preview }) {
  const t = useT();
  if (!preview.changes.length) return <Empty icon="check" title={t('projects.noChanges')}>{t('projects.noChangesHint')}</Empty>;
  return (
    <div className="table-wrap">
      <table className="table fixed top">
        <colgroup>
          <col style={{ width: '28%' }} />
          <col style={{ width: 110 }} />
          <col />
          <col style={{ width: 100 }} />
        </colgroup>
        <thead>
          <tr>
            <th>{t('common.name')}</th>
            <th>{t('change.action')}</th>
            <th>{t('common.value')}</th>
            <th>{t('projects.source')}</th>
          </tr>
        </thead>
        <tbody>
          {preview.changes.map((c) => (
            <tr key={c.name}>
              <td className="name" title={c.name}>
                {c.name}
              </td>
              <td>
                <Badge tone={KIND_TONE[c.kind]}>{t(`projects.kind.${c.kind}`)}</Badge>
              </td>
              <td className="value small">
                {c.sensitive ? <span className="mask faint">••••••••</span> : c.value == null ? <span className="faint">—</span> : <span className="mono break selectable">{c.value}</span>}
              </td>
              <td>
                <Badge>{t(`projects.from.${c.source}`)}</Badge>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ProjectHistory({ name }) {
  const t = useT();
  const lang = useLang();
  const { data } = useAsync(() => api.activityList({ target: name, limit: 100 }), [name]);
  if (!data) return <Loading />;
  if (!data.length) return <Empty icon="history" title={t('projects.noHistory')} />;
  return (
    <div className="list">
      {data.map((e) => (
        <div key={e.id} className="list-item">
          <span className={`dot ${e.status === 'ok' ? 'ok' : e.status === 'failed' ? 'danger' : 'warn'}`} />
          <span className="small">{t(`activity.actions.${e.action}`) !== `activity.actions.${e.action}` ? t(`activity.actions.${e.action}`) : e.action}</span>
          <span className="spacer" />
          <span className="xsmall faint">{formatDate(lang, e.ts)}</span>
        </div>
      ))}
    </div>
  );
}

function ProjectDetail({ id, onBack }) {
  const t = useT();
  const toast = useToast();
  const { navigate } = useNav();
  const { loading, error, data, reload } = useAsync(() => api.projectGet(id), [id]);
  const preview = useAsync(() => api.projectLaunchPreview(id), [id]);
  const [tab, setTab] = useState('env');
  const [editing, setEditing] = useState(false);
  const [launching, setLaunching] = useState(null);
  const [deleting, setDeleting] = useState(false);

  if (loading && !data) return <Loading label={t('common.loading')} />;
  if (error) return <div className="page"><Callout tone="danger">{errorText(t, error)}</Callout></div>;
  const p = data.project;

  return (
    <div className="page">
      <Button variant="ghost" size="sm" icon="chevronLeft" className="back-btn" onClick={() => onBack()}>
        {t('projects.title')}
      </Button>
      <PageHeader
        title={p.name}
        subtitle={p.description || t('profiles.noDescription')}
        actions={
          <>
            <Button icon="edit" onClick={() => setEditing(true)}>
              {t('common.edit')}
            </Button>
            {p.launch?.custom && (
              <Button icon="play" onClick={() => setLaunching('custom')} disabled={!data.directoryExists || !preview.data}>
                {t('projects.runCustom')}
              </Button>
            )}
            <Button variant="primary" icon="terminal" onClick={() => setLaunching('shell')} disabled={!data.directoryExists || !preview.data}>
              {t('projects.launchTerminal')}
            </Button>
          </>
        }
      />
      <div className="row" style={{ marginTop: -12, marginBottom: 16 }}>
        <Icon name="folder" size={14} />
        <span className="mono small selectable">{p.directory}</span>
        {!data.directoryExists && <Badge tone="danger">{t('projects.dirMissing')}</Badge>}
        {data.markers.map((m) => (
          <Badge key={m}>{t(`marker.${m}`)}</Badge>
        ))}
        <span className="spacer" />
        {p.profileName && (
          <Button size="sm" variant="ghost" icon="profile" onClick={() => navigate('profiles', { id: p.profileId })}>
            {p.profileName}
          </Button>
        )}
        <Button size="sm" variant="ghost" icon="trash" onClick={() => setDeleting(true)}>
          {t('common.delete')}
        </Button>
      </div>
      <Tabs
        value={tab}
        onChange={setTab}
        tabs={[
          { value: 'env', label: t('projects.launchEnv'), count: preview.data?.changes.length },
          { value: 'vars', label: t('projects.vars'), count: p.vars.length },
          { value: 'paths', label: t('profiles.pathChanges'), count: p.paths.length },
          { value: 'history', label: t('profiles.history') },
        ]}
      />
      {tab === 'env' && (
        <Card flush>
          <p className="small muted" style={{ padding: '16px 20px 0' }}>
            {t('projects.launchEnvHint')}
          </p>
          {preview.error && <div style={{ padding: 20 }}><Callout tone="danger">{errorText(t, preview.error)}</Callout></div>}
          {preview.data ? <EnvPreview preview={preview.data} /> : !preview.error && <Loading />}
        </Card>
      )}
      {tab === 'vars' && (
        <Card flush>
          <VarsTable vars={p.vars} reveal={(name) => api.projectRevealVar(id, name)} />
        </Card>
      )}
      {tab === 'paths' && (
        <Card flush>
          <PathsTable paths={p.paths} />
        </Card>
      )}
      {tab === 'history' && (
        <Card flush>
          <ProjectHistory name={p.name} />
        </Card>
      )}
      {editing && (
        <ProjectEditor
          project={p}
          onClose={() => setEditing(false)}
          onSaved={() => {
            setEditing(false);
            reload();
            preview.reload();
          }}
        />
      )}
      {launching && preview.data && <LaunchDialog project={p} preview={preview.data} mode={launching} onClose={() => setLaunching(null)} />}
      {deleting && (
        <ConfirmDialog
          title={t('projects.deleteTitle', { name: p.name })}
          danger
          confirmLabel={t('common.delete')}
          onCancel={() => setDeleting(false)}
          onConfirm={async () => {
            try {
              await api.projectDelete(id);
              toast.ok(t('projects.deleted'));
              onBack();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <p className="muted">{t('projects.deleteBody')}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}

function ProjectList({ onOpen }) {
  const t = useT();
  const { loading, error, data, reload } = useAsync(() => api.projectsList(), []);
  const [search, setSearch] = useState('');
  const [creating, setCreating] = useState(false);
  const q = search.trim().toLowerCase();
  const rows = (data || []).filter(
    (p) => !q || p.name.toLowerCase().includes(q) || p.directory.toLowerCase().includes(q) || p.tags.some((x) => x.toLowerCase().includes(q)),
  );
  return (
    <div className="page">
      <PageHeader
        title={t('projects.title')}
        subtitle={t('projects.subtitle')}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
            {t('projects.new')}
          </Button>
        }
      />
      <div className="toolbar">
        <SearchInput value={search} onChange={setSearch} placeholder={t('projects.search')} />
      </div>
      {loading && !data && <Loading label={t('common.loading')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      {data && !data.length && (
        <Card>
          <Empty
            icon="project"
            title={t('projects.emptyTitle')}
            action={
              <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
                {t('projects.new')}
              </Button>
            }
          >
            {t('projects.emptyBody')}
          </Empty>
        </Card>
      )}
      {rows.length > 0 && (
        <Card flush>
          <div className="list">
            {rows.map((p) => (
              <div key={p.id} className="list-item clickable" onClick={() => onOpen(p.id)} role="button" tabIndex={0} onKeyDown={(e) => e.key === 'Enter' && onOpen(p.id)}>
                <Icon name="project" size={18} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="row">
                    <strong>{p.name}</strong>
                    {!p.directoryExists && <Badge tone="danger">{t('projects.dirMissing')}</Badge>}
                  </div>
                  <div className="mono xsmall faint truncate">{p.directory}</div>
                </div>
                {p.profileName && <Badge icon="profile">{p.profileName}</Badge>}
                <Badge>{t('profiles.varCount', { count: p.varCount })}</Badge>
                <Icon name="chevronRight" size={14} />
              </div>
            ))}
          </div>
        </Card>
      )}
      {creating && (
        <ProjectEditor
          onClose={() => setCreating(false)}
          onSaved={(id) => {
            setCreating(false);
            reload();
            onOpen(id);
          }}
        />
      )}
    </div>
  );
}

export default function Projects({ params }) {
  const { navigate } = useNav();
  const open = (id) => navigate('projects', id ? { id } : null);
  if (params?.id) return <ProjectDetail key={params.id} id={params.id} onBack={open} />;
  return <ProjectList onOpen={open} />;
}
