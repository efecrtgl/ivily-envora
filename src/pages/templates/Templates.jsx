import { useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog, Modal } from '../../components/ui/Modal.jsx';
import { Badge, Button, Callout, Card, Field, Loading, PageHeader } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import ExportDialog from '../../components/transfer/ExportDialog.jsx';
import ProfileEditor from '../profiles/ProfileEditor.jsx';
import ProjectEditor from '../projects/ProjectEditor.jsx';
import TemplateEditor from './TemplateEditor.jsx';

const ROLE_TONE = { required: 'danger', recommended: 'ok', optional: undefined };

/** Şablon değişkenlerini profil değişkenlerine dönüştürür. */
export function templateVars(tpl, scopeOverride) {
  return tpl.vars.map((v) => ({ name: v.name, value: v.value, sensitive: false, kind: v.kind ?? null, scope: scopeOverride || v.scope, action: 'set' }));
}

/** Şablonu mevcut değişkenlerle birleştirir: zaten olan adlar korunur, eksikler eklenir. */
export function mergeVars(existing, added) {
  const have = new Set(existing.map((v) => v.name.toUpperCase()));
  return [...existing, ...added.filter((v) => !have.has(v.name.toUpperCase()))];
}

export function mergePaths(existing, added) {
  const key = (p) => `${p.scope}|${p.entry.toLowerCase()}`;
  const have = new Set(existing.map(key));
  return [...existing, ...added.filter((p) => !have.has(key(p)))];
}

function TemplateDetail({ tpl, onClose, onChanged }) {
  const t = useT();
  const toast = useToast();
  const [action, setAction] = useState(null); // 'newProfile' | 'profile' | 'project' | 'edit' | 'duplicate' | 'delete' | 'export'
  const [target, setTarget] = useState(null);
  const [name, setName] = useState('');
  const profiles = useAsync(() => api.profilesList(false), []);
  const projects = useAsync(() => api.projectsList(), []);

  const pickProfile = async (id) => {
    try {
      const p = await api.profileGet(Number(id));
      setTarget({ ...p, vars: mergeVars(p.vars, templateVars(tpl)), paths: mergePaths(p.paths, tpl.paths) });
      setAction('profile');
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };
  const pickProject = async (id) => {
    try {
      const v = await api.projectGet(Number(id));
      const p = v.project;
      setTarget({ ...p, vars: mergeVars(p.vars, templateVars(tpl, 'process')), paths: mergePaths(p.paths, tpl.paths.map((x) => ({ ...x, scope: 'process' }))) });
      setAction('project');
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  if (action === 'newProfile') {
    return (
      <ProfileEditor
        initial={{ name: tpl.name, description: tpl.description, category: tpl.category, tags: [], vars: templateVars(tpl), paths: tpl.paths, metadata: { template: tpl.id } }}
        onClose={() => setAction(null)}
        onSaved={() => {
          setAction(null);
          onClose();
        }}
      />
    );
  }
  if (action === 'profile' && target) return <ProfileEditor profile={target} onClose={() => setAction(null)} onSaved={() => onClose()} />;
  if (action === 'project' && target) return <ProjectEditor project={target} onClose={() => setAction(null)} onSaved={() => onClose()} />;
  if (action === 'edit') return <TemplateEditor template={tpl} onClose={() => setAction(null)} onSaved={() => { setAction(null); onChanged(); onClose(); }} />;
  if (action === 'export') return <ExportDialog preset={{ kind: 'template', id: tpl.id }} onClose={() => setAction(null)} />;

  return (
    <Modal
      wide
      title={tpl.name}
      onClose={onClose}
      footer={
        <>
          {!tpl.builtin && (
            <>
              <Button variant="ghost" icon="trash" onClick={() => setAction('delete')}>
                {t('common.delete')}
              </Button>
              <Button variant="ghost" icon="edit" onClick={() => setAction('edit')}>
                {t('common.edit')}
              </Button>
            </>
          )}
          <Button variant="ghost" icon="copy" onClick={() => { setName(`${tpl.name} (2)`); setAction('duplicate'); }}>
            {t('common.duplicate')}
          </Button>
          <Button variant="ghost" icon="download" onClick={() => setAction('export')}>
            {t('common.export')}
          </Button>
          <span className="spacer" />
          <Button variant="primary" icon="plus" onClick={() => setAction('newProfile')}>
            {t('templates.createProfile')}
          </Button>
        </>
      }
    >
      <div className="stack">
        <div className="row">
          {tpl.builtin ? <Badge tone="info">{t('templates.builtin')}</Badge> : <Badge>{t('templates.user')}</Badge>}
          {tpl.category && <Badge>{tpl.category}</Badge>}
        </div>
        <p className="muted small">{tpl.description}</p>
        <Callout tone="info">{t('templates.recommendationNote')}</Callout>
        {tpl.vars.length > 0 && (
          <div className="list" style={{ border: '1px solid var(--border)', borderRadius: 10 }}>
            {tpl.vars.map((v) => (
              <div key={v.name} className="list-item" style={{ alignItems: 'flex-start' }}>
                <div style={{ width: 230, flex: 'none' }}>
                  <div className="mono small" style={{ fontWeight: 600 }}>
                    {v.name}
                  </div>
                  <div className="row" style={{ marginTop: 4 }}>
                    <Badge tone={ROLE_TONE[v.role]}>{t(`templates.role.${v.role}`)}</Badge>
                    <ScopeBadge scope={v.scope} />
                  </div>
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="mono small break">{v.value || <span className="faint">{t('templates.fillIn')}</span>}</div>
                  {v.note && <div className="xsmall muted" style={{ marginTop: 2 }}>{v.note}</div>}
                </div>
              </div>
            ))}
          </div>
        )}
        {tpl.paths.length > 0 && (
          <div>
            <div className="field-label" style={{ marginBottom: 6 }}>
              {t('profiles.pathChanges')}
            </div>
            {tpl.paths.map((p, i) => (
              <div key={i} className="row small" style={{ marginBottom: 4 }}>
                <Badge tone="ok">{t(`envset.${p.action}`)}</Badge>
                <span className="mono">{p.entry}</span>
                <ScopeBadge scope={p.scope} />
              </div>
            ))}
          </div>
        )}
        <div className="grid cols-2">
          <Field label={t('templates.applyToProfile')}>
            <select className="select" value="" onChange={(e) => e.target.value && pickProfile(e.target.value)}>
              <option value="">{t('templates.choose')}</option>
              {(profiles.data || []).map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label={t('templates.applyToProject')}>
            <select className="select" value="" onChange={(e) => e.target.value && pickProject(e.target.value)}>
              <option value="">{t('templates.choose')}</option>
              {(projects.data || []).map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <p className="xsmall faint">{t('templates.applyNote')}</p>
      </div>
      {action === 'duplicate' && (
        <ConfirmDialog
          title={t('common.duplicate')}
          confirmLabel={t('common.duplicate')}
          disabled={!name.trim()}
          onCancel={() => setAction(null)}
          onConfirm={async () => {
            try {
              await api.templateDuplicate(tpl.id, name);
              toast.ok(t('templates.duplicated'));
              onChanged();
              onClose();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <Field label={t('common.name')}>
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} maxLength={100} />
          </Field>
        </ConfirmDialog>
      )}
      {action === 'delete' && (
        <ConfirmDialog
          danger
          title={t('templates.deleteTitle', { name: tpl.name })}
          confirmLabel={t('common.delete')}
          onCancel={() => setAction(null)}
          onConfirm={async () => {
            try {
              await api.templateDelete(tpl.id);
              onChanged();
              onClose();
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <p className="muted">{t('templates.deleteBody')}</p>
        </ConfirmDialog>
      )}
    </Modal>
  );
}

export default function Templates() {
  const t = useT();
  const { loading, error, data, reload } = useAsync(() => api.templatesList(), []);
  const [open, setOpen] = useState(null);
  const [creating, setCreating] = useState(false);

  return (
    <div className="page">
      <PageHeader
        title={t('templates.title')}
        subtitle={t('templates.subtitle')}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
            {t('templates.new')}
          </Button>
        }
      />
      {loading && !data && <Loading label={t('common.loading')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      <div className="grid cols-3">
        {(data || []).map((tpl) => (
          <button key={tpl.id} type="button" className="card profile-card" onClick={() => setOpen(tpl)}>
            <div className="row between" style={{ marginBottom: 6 }}>
              <h3 className="truncate" style={{ fontSize: 'var(--fs-lg)' }}>
                {tpl.name}
              </h3>
              {tpl.builtin ? <Badge tone="info">{t('templates.builtin')}</Badge> : <Badge>{t('templates.user')}</Badge>}
            </div>
            <p className="small muted clamp-2">{tpl.description}</p>
            <div className="row xsmall faint" style={{ marginTop: 10 }}>
              {t('profiles.counts', { vars: tpl.vars.length, paths: tpl.paths.length })}
            </div>
          </button>
        ))}
      </div>
      {open && <TemplateDetail tpl={open} onClose={() => setOpen(null)} onChanged={reload} />}
      {creating && (
        <TemplateEditor
          onClose={() => setCreating(false)}
          onSaved={() => {
            setCreating(false);
            reload();
          }}
        />
      )}
    </div>
  );
}
