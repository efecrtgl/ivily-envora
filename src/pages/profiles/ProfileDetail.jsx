import { useState } from 'react';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog, Modal } from '../../components/ui/Modal.jsx';
import { Badge, Button, Callout, Card, Empty, Field, Loading, PageHeader, Tabs } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import SecretValue from '../../components/env/SecretValue.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';
import ProfileEditor from './ProfileEditor.jsx';
import ExportDialog from '../../components/transfer/ExportDialog.jsx';

const SEVERITY_TONE = { error: 'danger', warning: 'warn', info: 'info' };

export function FindingList({ findings }) {
  const t = useT();
  if (!findings.length) return <Callout tone="info" icon="check">{t('profileCheck.clean')}</Callout>;
  return (
    <div className="list">
      {findings.map((f, i) => (
        <div key={i} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
          <Badge tone={SEVERITY_TONE[f.severity]}>{t(`severity.${f.severity}`)}</Badge>
          <span className="mono small truncate" style={{ maxWidth: 260 }} title={f.target}>
            {f.target}
          </span>
          <span className="small muted">{t(f.code.startsWith('issue.') ? f.code : `profileCheck.${f.code}`)}</span>
        </div>
      ))}
    </div>
  );
}

export function VarsTable({ vars, reveal }) {
  const t = useT();
  if (!vars.length) return <Empty icon="variable" title={t('profiles.noVars')} />;
  return (
    <div className="table-wrap">
      <table className="table fixed">
        <colgroup>
          <col style={{ width: '28%' }} />
          <col />
          <col style={{ width: 118 }} />
          <col style={{ width: 130 }} />
        </colgroup>
        <thead>
          <tr>
            <th>{t('common.name')}</th>
            <th>{t('common.value')}</th>
            <th>{t('common.scope')}</th>
            <th>{t('envset.action')}</th>
          </tr>
        </thead>
        <tbody>
          {vars.map((v) => (
            <tr key={v.name}>
              <td className="name" title={v.name}>
                {v.name}
              </td>
              <td className="value">
                {v.action === 'unset' ? (
                  <span className="faint">—</span>
                ) : (
                  <SecretValue v={{ ...v, masked: v.value == null, scope: v.scope, kind: v.kind || 'string' }} reveal={reveal ? () => reveal(v.name) : null} />
                )}
              </td>
              <td>
                <ScopeBadge scope={v.scope} />
              </td>
              <td>
                <Badge tone={v.action === 'unset' ? 'danger' : undefined}>{t(`envset.${v.action}`)}</Badge>
                {v.kind && <Badge>{t(`kind.${v.kind}`)}</Badge>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function PathsTable({ paths }) {
  const t = useT();
  if (!paths.length) return <Empty icon="path" title={t('profiles.noPaths')} />;
  return (
    <div className="list">
      {paths.map((p, i) => (
        <div key={i} className="list-item">
          <Badge tone={p.action === 'remove' ? 'danger' : 'ok'}>{t(`envset.${p.action}`)}</Badge>
          <span className="mono small break selectable" style={{ flex: 1 }}>
            {p.entry}
          </span>
          <ScopeBadge scope={p.scope} />
        </div>
      ))}
    </div>
  );
}

function History({ id, onRestored }) {
  const t = useT();
  const lang = useLang();
  const toast = useToast();
  const revs = useAsync(() => api.profileRevisions(id), [id]);
  const acts = useAsync(() => api.profileActivations(id), [id]);
  const [viewing, setViewing] = useState(null);
  const [restoring, setRestoring] = useState(null);

  return (
    <div className="grid cols-2">
      <Card title={t('profiles.revisions')} icon="history">
        {revs.data?.length === 0 && <p className="small faint">{t('profiles.noRevisions')}</p>}
        <div className="list">
          {(revs.data || []).map((r) => (
            <div key={r.id} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
              <Badge>v{r.version}</Badge>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div className="small truncate">{r.note || t('profiles.noNote')}</div>
                <div className="xsmall faint">
                  {formatDate(lang, r.createdAt)} · {t('profiles.varCount', { count: r.varCount })}
                </div>
              </div>
              <Button size="sm" variant="ghost" icon="eye" onClick={() => api.profileRevisionGet(id, r.id).then(setViewing).catch((e) => toast.error(errorText(t, e)))}>
                {t('common.details')}
              </Button>
              <Button size="sm" variant="ghost" icon="undo" onClick={() => setRestoring(r)}>
                {t('profiles.restore')}
              </Button>
            </div>
          ))}
        </div>
      </Card>
      <Card title={t('profiles.activations')} icon="play">
        {acts.data?.length === 0 && <p className="small faint">{t('profiles.noActivations')}</p>}
        <div className="list">
          {(acts.data || []).map((a) => (
            <div key={a.id} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
              <span className={`dot ${a.deactivatedAt ? '' : 'ok'}`} />
              <ScopeBadge scope={a.scope} />
              <div className="xsmall muted" style={{ flex: 1 }}>
                {formatDate(lang, a.appliedAt)}
                {a.deactivatedAt && ` → ${formatDate(lang, a.deactivatedAt)}`}
              </div>
            </div>
          ))}
        </div>
      </Card>
      {viewing && (
        <Modal wide title={`${viewing.name} · v${viewing.version}`} onClose={() => setViewing(null)} footer={<Button onClick={() => setViewing(null)}>{t('common.close')}</Button>}>
          <div className="stack">
            <VarsTable vars={viewing.vars} reveal={null} />
            <PathsTable paths={viewing.paths} />
          </div>
        </Modal>
      )}
      {restoring && (
        <ConfirmDialog
          title={t('profiles.restoreTitle', { version: restoring.version })}
          confirmLabel={t('profiles.restore')}
          onCancel={() => setRestoring(null)}
          onConfirm={async () => {
            try {
              await api.profileRestoreRevision(id, restoring.id);
              toast.ok(t('profiles.restored'));
              setRestoring(null);
              revs.reload();
              onRestored();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <p className="muted">{t('profiles.restoreBody')}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}

export default function ProfileDetail({ id, onBack }) {
  const t = useT();
  const toast = useToast();
  const { loading, error, data: p, reload } = useAsync(() => api.profileGet(id), [id]);
  const findings = useAsync(() => api.profileValidate(id), [id]);
  const [tab, setTab] = useState('vars');
  const [editing, setEditing] = useState(false);
  const [flows, setFlows] = useState([]); // sıradaki değişiklik akışları
  const [dialog, setDialog] = useState(null); // 'duplicate' | 'rename' | 'delete'
  const [text, setText] = useState('');
  const [exporting, setExporting] = useState(false);

  const refresh = () => {
    reload();
    findings.reload();
  };

  const activate = async () => {
    try {
      const plans = await api.profileActivationPlans(id);
      if (!plans.length) {
        toast.info(t('profiles.nothingToActivate'));
        return;
      }
      setFlows(plans.map((plan, i) => ({ plan, context: { kind: 'profileActivate', profileId: id }, title: t('profiles.activateStep', { step: i + 1, total: plans.length, scope: t(`scope.${plan.scope}`) }) })));
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const deactivate = async (scope) => {
    try {
      const plan = await api.profileDeactivationPlan(id, scope);
      setFlows([{ plan, context: { kind: 'profileDeactivate', profileId: id }, title: t('profiles.deactivateTitle', { scope: t(`scope.${scope}`) }) }]);
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const runDialog = async () => {
    try {
      if (dialog === 'duplicate') {
        const nid = await api.profileDuplicate(id, text);
        toast.ok(t('profiles.duplicated'));
        setDialog(null);
        onBack(nid);
        return;
      }
      if (dialog === 'rename') await api.profileRename(id, text);
      if (dialog === 'delete') {
        await api.profileDelete(id);
        toast.ok(t('profiles.deleted'));
        onBack();
        return;
      }
      setDialog(null);
      refresh();
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  if (loading && !p) return <Loading label={t('common.loading')} />;
  if (error) return <div className="page"><Callout tone="danger">{errorText(t, error)}</Callout></div>;
  if (!p) return null;

  const archived = p.status === 'archived';
  const errorsCount = (findings.data || []).filter((f) => f.severity === 'error').length;

  return (
    <div className="page">
      <Button variant="ghost" size="sm" icon="chevronLeft" className="back-btn" onClick={() => onBack()}>
        {t('profiles.title')}
      </Button>
      <PageHeader
        title={p.name}
        subtitle={p.description || t('profiles.noDescription')}
        actions={
          <>
            <Button icon="edit" onClick={() => setEditing(true)}>
              {t('common.edit')}
            </Button>
            <Button variant="primary" icon="play" onClick={activate} disabled={archived || errorsCount > 0} title={errorsCount ? t('profiles.fixBeforeActivate') : undefined}>
              {t('profiles.activate')}
            </Button>
          </>
        }
      />
      <div className="row" style={{ marginTop: -12, marginBottom: 20 }}>
        <Badge>v{p.version}</Badge>
        {p.category && <Badge>{p.category}</Badge>}
        {p.tags.map((tag) => (
          <Badge key={tag} icon="tag">
            {tag}
          </Badge>
        ))}
        {archived && <Badge icon="archive">{t('profiles.archived')}</Badge>}
        <span className="spacer" />
        <Button size="sm" variant="ghost" icon="copy" onClick={() => { setText(`${p.name} (2)`); setDialog('duplicate'); }}>
          {t('common.duplicate')}
        </Button>
        <Button size="sm" variant="ghost" icon="tag" onClick={() => { setText(p.name); setDialog('rename'); }}>
          {t('common.rename')}
        </Button>
        <Button size="sm" variant="ghost" icon="archive" onClick={() => api.profileSetStatus(id, archived ? 'active' : 'archived').then(refresh).catch((e) => toast.error(errorText(t, e)))}>
          {archived ? t('profiles.unarchive') : t('profiles.archive')}
        </Button>
        <Button size="sm" variant="ghost" icon="download" onClick={() => setExporting(true)}>
          {t('common.export')}
        </Button>
        <Button size="sm" variant="ghost" icon="trash" onClick={() => setDialog('delete')}>
          {t('common.delete')}
        </Button>
      </div>

      <ActiveScopes id={id} onDeactivate={deactivate} version={p.version} />

      <Tabs
        value={tab}
        onChange={setTab}
        tabs={[
          { value: 'vars', label: t('profiles.variables'), count: p.vars.length },
          { value: 'paths', label: t('profiles.pathChanges'), count: p.paths.length },
          { value: 'check', label: t('profiles.validation'), count: findings.data?.length },
          { value: 'history', label: t('profiles.history') },
        ]}
      />
      {tab === 'vars' && (
        <Card flush>
          <VarsTable vars={p.vars} reveal={(name) => api.profileRevealVar(id, name)} />
        </Card>
      )}
      {tab === 'paths' && (
        <Card flush>
          <PathsTable paths={p.paths} />
        </Card>
      )}
      {tab === 'check' && <Card>{findings.data ? <FindingList findings={findings.data} /> : <Loading />}</Card>}
      {tab === 'history' && <History id={id} onRestored={refresh} />}

      {exporting && <ExportDialog preset={{ kind: 'profile', id }} onClose={() => setExporting(false)} />}
      {editing && (
        <ProfileEditor
          profile={p}
          onClose={() => setEditing(false)}
          onSaved={() => {
            setEditing(false);
            refresh();
          }}
        />
      )}
      {flows.length > 0 && (
        <ChangeFlow
          key={flows.length}
          plan={flows[0].plan}
          context={flows[0].context}
          title={flows[0].title}
          onClose={() => setFlows((f) => f.slice(1))}
          onDone={refresh}
        />
      )}
      {dialog && (
        <ConfirmDialog
          title={t(`profiles.${dialog}Title`, { name: p.name })}
          danger={dialog === 'delete'}
          confirmLabel={t(dialog === 'delete' ? 'common.delete' : dialog === 'rename' ? 'common.rename' : 'common.duplicate')}
          disabled={dialog !== 'delete' && !text.trim()}
          onCancel={() => setDialog(null)}
          onConfirm={runDialog}
        >
          {dialog === 'delete' ? (
            <p className="muted">{t('profiles.deleteBody')}</p>
          ) : (
            <Field label={t('common.name')}>
              <input className="input" value={text} onChange={(e) => setText(e.target.value)} maxLength={100} />
            </Field>
          )}
        </ConfirmDialog>
      )}
    </div>
  );
}

function ActiveScopes({ id, onDeactivate, version }) {
  const t = useT();
  const { data } = useAsync(() => api.profileActivations(id), [id, version]);
  const active = (data || []).filter((a) => !a.deactivatedAt);
  if (!active.length) return null;
  return (
    <Callout tone="info" icon="play">
      <div className="row">
        <span>{t('profiles.activeNotice')}</span>
        {active.map((a) => (
          <span key={a.id} className="row">
            <ScopeBadge scope={a.scope} />
            <Button size="sm" variant="ghost" icon="undo" onClick={() => onDeactivate(a.scope)}>
              {t('profiles.deactivate')}
            </Button>
          </span>
        ))}
      </div>
    </Callout>
  );
}
