import { useCallback, useEffect, useState } from 'react';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { useT, LANGUAGES, errorText, useLang } from '../i18n/index.jsx';
import { useSettings } from '../app/settings.jsx';
import { PAGES } from '../app/nav.js';
import { api, inTauri } from '../lib/api.js';
import { formatBytes } from '../lib/format.js';
import { useToast } from '../components/ui/Toast.jsx';
import { ConfirmDialog, Modal } from '../components/ui/Modal.jsx';
import { Badge, Button, Callout, Card, PageHeader, Segmented, Toggle } from '../components/ui/primitives.jsx';

const START_PAGES = ['dashboard', 'variables', 'path', 'profiles', 'projects'];
const DASHBOARD_SECTIONS = ['overview', 'pathHealth', 'analysis', 'security', 'profiles', 'projects', 'tools', 'activity'];

/** Görünür bölümleri sıralı tutar; gizli olanlar listede yer almaz. */
export function toggleSection(list, id, on) {
  if (on) return list.includes(id) ? list : [...list, id];
  return list.filter((x) => x !== id);
}

export function moveSection(list, id, delta) {
  const i = list.indexOf(id);
  const j = i + delta;
  if (i < 0 || j < 0 || j >= list.length) return list;
  const next = [...list];
  [next[i], next[j]] = [next[j], next[i]];
  return next;
}

function DashboardSection({ sections, onChange }) {
  const t = useT();
  const hidden = DASHBOARD_SECTIONS.filter((s) => !sections.includes(s));
  return (
    <div className="list">
      {[...sections, ...hidden].map((id) => {
        const on = sections.includes(id);
        const i = sections.indexOf(id);
        return (
          <div key={id} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
            <Toggle label={t(`dashboard.section.${id}`)} checked={on} onChange={(v) => onChange(toggleSection(sections, id, v))} />
            <span className="small" style={{ flex: 1, opacity: on ? 1 : 0.55 }}>
              {t(`dashboard.section.${id}`)}
            </span>
            {on && (
              <>
                <Button size="sm" variant="ghost" iconOnly icon="arrowUp" title={t('envset.moveUp')} disabled={i === 0} onClick={() => onChange(moveSection(sections, id, -1))} />
                <Button size="sm" variant="ghost" iconOnly icon="arrowDown" title={t('envset.moveDown')} disabled={i === sections.length - 1} onClick={() => onChange(moveSection(sections, id, 1))} />
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}

/** Etiket + açıklama solda, denetim sağda olan ayar satırı. */
export function SettingRow({ label, hint, children }) {
  return (
    <div className="setting-row">
      <div>
        <div className="setting-label">{label}</div>
        {hint && <div className="field-hint">{hint}</div>}
      </div>
      <div className="setting-control">{children}</div>
    </div>
  );
}

export function NumberSetting({ value, min, max, onCommit, suffix }) {
  const [draft, setDraft] = useState(String(value));
  useEffect(() => setDraft(String(value)), [value]);
  const commit = () => {
    const n = Number(draft);
    if (Number.isInteger(n) && n >= min && n <= max) {
      if (n !== value) onCommit(n);
    } else setDraft(String(value));
  };
  return (
    <div className="row">
      <input
        className="input"
        style={{ width: 96 }}
        type="number"
        min={min}
        max={max}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === 'Enter' && commit()}
      />
      {suffix && <span className="muted small">{suffix}</span>}
    </div>
  );
}

function OverridesSection() {
  const t = useT();
  const toast = useToast();
  const [items, setItems] = useState(null);
  const load = useCallback(() => {
    if (inTauri()) api.sensitivityList().then(setItems).catch(() => setItems([]));
    else setItems([]);
  }, []);
  useEffect(load, [load]);
  const clear = async (name) => {
    try {
      await api.sensitivitySet(name, null);
      load();
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };
  if (!items) return null;
  if (!items.length) return <p className="small faint">{t('settings.noOverrides')}</p>;
  return (
    <div className="list">
      {items.map((o) => (
        <div key={o.name} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
          <span className="mono small">{o.name}</span>
          <Badge tone={o.sensitive ? 'warn' : undefined}>{o.sensitive ? t('vars.sensYes') : t('vars.sensNo')}</Badge>
          <span className="spacer" />
          <Button size="sm" variant="ghost" icon="undo" onClick={() => clear(o.name)}>
            {t('settings.clearOverride')}
          </Button>
        </div>
      ))}
    </div>
  );
}

function DataSection() {
  const t = useT();
  const toast = useToast();
  const [status, setStatus] = useState(null);

  const load = useCallback(() => {
    if (inTauri()) api.dbStatus().then(setStatus).catch(() => setStatus(null));
  }, []);
  useEffect(load, [load]);

  const backup = async () => {
    const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-');
    const dest = await saveDialog({
      defaultPath: `envora-yedek-${stamp}.db`,
      filters: [{ name: 'SQLite', extensions: ['db'] }],
    });
    if (!dest) return;
    try {
      await api.dbBackup(dest);
      toast.ok(t('settings.backupDone'));
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const [restore, setRestore] = useState(null); // { path, info }
  const [pending, setPending] = useState(false);
  useEffect(() => {
    if (inTauri()) api.dbRestorePending().then(setPending).catch(() => {});
  }, []);

  const pickRestore = async () => {
    const path = await openDialog({ multiple: false, filters: [{ name: 'SQLite', extensions: ['db'] }] });
    if (!path) return;
    try {
      setRestore({ path, info: await api.dbBackupInspect(path) });
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const scheduleRestore = async () => {
    try {
      await api.dbRestoreSchedule(restore.path);
      setRestore(null);
      setPending(true);
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  return (
    <>
    {pending && (
      <Callout tone="warn" icon="history">
        <div className="row">
          <span>{t('settings.restorePending')}</span>
          <Button size="sm" variant="primary" icon="refresh" onClick={() => api.appRestart()}>
            {t('settings.restartNow')}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => api.dbRestoreCancel().then(() => setPending(false))}>
            {t('common.cancel')}
          </Button>
        </div>
      </Callout>
    )}
    <dl className="kv" style={{ marginTop: pending ? 12 : 0 }}>
      <dt>{t('settings.dbSchema')}</dt>
      <dd>{status ? `v${status.schemaVersion}` : '—'}</dd>
      <dt>{t('settings.dbIntegrity')}</dt>
      <dd>
        {status ? (
          <Badge tone={status.integrityOk ? 'ok' : 'danger'}>
            {status.integrityOk ? t('settings.integrityOk') : t('settings.integrityBad')}
          </Badge>
        ) : (
          '—'
        )}
      </dd>
      <dt>{t('settings.dbSize')}</dt>
      <dd>{status ? formatBytes(status.sizeBytes) : '—'}</dd>
      <dt />
      <dd className="row">
        <Button size="sm" icon="download" onClick={backup} disabled={!inTauri()}>
          {t('settings.backupDb')}
        </Button>
        <Button size="sm" icon="upload" onClick={pickRestore} disabled={!inTauri()}>
          {t('settings.restoreDb')}
        </Button>
        <Button size="sm" variant="ghost" icon="refresh" onClick={load} disabled={!inTauri()}>
          {t('common.refresh')}
        </Button>
      </dd>
    </dl>
    {restore && (
      <ConfirmDialog
        title={t('settings.restoreTitle')}
        confirmLabel={t('settings.restoreConfirm')}
        danger
        disabled={!restore.info.restorable}
        onCancel={() => setRestore(null)}
        onConfirm={scheduleRestore}
      >
        <div className="stack">
          <dl className="kv">
            <dt>{t('settings.dbSchema')}</dt>
            <dd>
              v{restore.info.schemaVersion} / v{restore.info.latestVersion}
            </dd>
            <dt>{t('settings.dbIntegrity')}</dt>
            <dd>
              <Badge tone={restore.info.integrityOk ? 'ok' : 'danger'}>{restore.info.integrityOk ? t('settings.integrityOk') : t('settings.integrityBad')}</Badge>
            </dd>
            <dt>{t('nav.profiles')}</dt>
            <dd>{restore.info.profiles ?? '—'}</dd>
            <dt>{t('nav.projects')}</dt>
            <dd>{restore.info.projects ?? '—'}</dd>
            <dt>{t('nav.snapshots')}</dt>
            <dd>{restore.info.snapshots ?? '—'}</dd>
          </dl>
          {restore.info.restorable ? (
            <Callout tone="warn">{t('settings.restoreBody')}</Callout>
          ) : (
            <Callout tone="danger">{t('settings.restoreInvalid')}</Callout>
          )}
        </div>
      </ConfirmDialog>
    )}
    </>
  );
}

export default function Settings() {
  const t = useT();
  const lang = useLang();
  const toast = useToast();
  const { settings: s, meta, set, reset } = useSettings();
  const [info, setInfo] = useState(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const [report, setReport] = useState(null);

  useEffect(() => {
    api.appInfo().then(setInfo).catch(() => setInfo(null));
  }, []);

  const change = (path) => (value) => set(path, value).catch((e) => toast.error(errorText(t, e)));
  const startPages = START_PAGES.filter((p) => PAGES[p]);

  return (
    <div className="page">
      <PageHeader title={t('settings.title')} subtitle={t('settings.subtitle')} />
      <div className="stack">
        {meta.fixed.length > 0 && (
          <Callout tone="warn" title={t('settings.fixedTitle')}>
            {t('settings.fixedBody', { fields: meta.fixed.join(', ') })}
          </Callout>
        )}

        <Card title={t('settings.general')} icon="settings">
          <SettingRow label={t('settings.language')} hint={t('settings.languageHint')}>
            <Segmented label={t('settings.language')} value={s.general.language} onChange={change('general.language')} options={LANGUAGES} />
          </SettingRow>
          <SettingRow label={t('settings.startPage')} hint={t('settings.startPageHint')}>
            <select className="select" style={{ width: 200 }} value={s.general.startPage} onChange={(e) => change('general.startPage')(e.target.value)}>
              {startPages.map((p) => (
                <option key={p} value={p}>
                  {t(`nav.${p}`)}
                </option>
              ))}
            </select>
          </SettingRow>
        </Card>

        <Card title={t('settings.appearance')} icon="sun">
          <SettingRow label={t('theme.label')} hint={t('settings.themeHint')}>
            <Segmented
              label={t('theme.label')}
              value={s.appearance.theme}
              onChange={change('appearance.theme')}
              options={['system', 'dark', 'light'].map((v) => ({ value: v, label: t(`theme.${v}`) }))}
            />
          </SettingRow>
          <SettingRow label={t('settings.density')}>
            <Segmented
              label={t('settings.density')}
              value={s.appearance.density}
              onChange={change('appearance.density')}
              options={[
                { value: 'comfortable', label: t('settings.densityComfortable') },
                { value: 'compact', label: t('settings.densityCompact') },
              ]}
            />
          </SettingRow>
          <SettingRow label={t('settings.reduceMotion')} hint={t('settings.reduceMotionHint')}>
            <Toggle label={t('settings.reduceMotion')} checked={s.appearance.reduceMotion} onChange={change('appearance.reduceMotion')} />
          </SettingRow>
        </Card>

        <Card title={t('settings.dashboard')} icon="dashboard" desc={t('settings.dashboardHint')}>
          <DashboardSection sections={s.dashboard.sections} onChange={change('dashboard.sections')} />
        </Card>

        <Card title={t('settings.security')} icon="shield">
          <SettingRow label={t('settings.autoHide')} hint={t('settings.autoHideHint')}>
            <NumberSetting value={s.security.autoHideSeconds} min={0} max={3600} suffix={t('settings.seconds')} onCommit={change('security.autoHideSeconds')} />
          </SettingRow>
          <SettingRow label={t('settings.warnOnCopy')}>
            <Toggle label={t('settings.warnOnCopy')} checked={s.security.warnOnCopy} onChange={change('security.warnOnCopy')} />
          </SettingRow>
          <SettingRow label={t('settings.clipboardClear')} hint={t('settings.clipboardClearHint')}>
            <NumberSetting value={s.security.clipboardClearSeconds} min={0} max={600} suffix={t('settings.seconds')} onCommit={change('security.clipboardClearSeconds')} />
          </SettingRow>
          <div style={{ paddingTop: 12 }}>
            <div className="setting-label">{t('settings.overrides')}</div>
            <div className="field-hint" style={{ marginBottom: 6 }}>{t('settings.overridesHint')}</div>
            <OverridesSection />
          </div>
        </Card>

        <Card title={t('settings.activity')} icon="activity">
          <SettingRow label={t('settings.activityEnabled')} hint={t('settings.activityEnabledHint')}>
            <Toggle label={t('settings.activityEnabled')} checked={s.activity.enabled} onChange={change('activity.enabled')} />
          </SettingRow>
          <SettingRow label={t('settings.retention')} hint={t('settings.retentionHint')}>
            <NumberSetting value={s.activity.retentionDays} min={1} max={3650} suffix={t('settings.days')} onCommit={change('activity.retentionDays')} />
          </SettingRow>
        </Card>

        <Card title={t('settings.backup')} icon="snapshot">
          <SettingRow label={t('settings.autoSnapshot')} hint={t('settings.autoSnapshotHint')}>
            <Toggle label={t('settings.autoSnapshot')} checked={s.backup.autoSnapshotBeforeChanges} onChange={change('backup.autoSnapshotBeforeChanges')} />
          </SettingRow>
          <SettingRow label={t('settings.maxAutoSnapshots')} hint={t('settings.maxAutoSnapshotsHint')}>
            <NumberSetting value={s.backup.maxAutoSnapshots} min={5} max={1000} onCommit={change('backup.maxAutoSnapshots')} />
          </SettingRow>
          <SettingRow label={t('settings.excludeInSnapshots')} hint={t('settings.excludeInSnapshotsHint')}>
            <Toggle label={t('settings.excludeInSnapshots')} checked={s.security.excludeSensitiveInSnapshots} onChange={change('security.excludeSensitiveInSnapshots')} />
          </SettingRow>
        </Card>

        <Card title={t('settings.data')} icon="database" desc={t('settings.dataHint')}>
          <DataSection />
        </Card>

        <Card title={t('settings.about')} icon="info">
          <dl className="kv">
            <dt>{t('common.version')}</dt>
            <dd>{info?.version || __APP_VERSION__}</dd>
            <dt>{t('settings.platform')}</dt>
            <dd>{info ? `${info.platform} / ${info.arch}` : '—'}</dd>
            <dt>{t('settings.language')}</dt>
            <dd>{lang}</dd>
            <dt>{t('settings.license')}</dt>
            <dd>MIT</dd>
          </dl>
          <div className="row" style={{ marginTop: 16 }}>
            <Button
              size="sm"
              icon="info"
              disabled={!inTauri()}
              onClick={() => api.diagnosticsReport().then(setReport, (e) => toast.error(errorText(t, e)))}
            >
              {t('settings.diagnostics')}
            </Button>
            <Button size="sm" variant="danger" icon="undo" onClick={() => setConfirmReset(true)}>
              {t('settings.reset')}
            </Button>
          </div>
        </Card>
      </div>

      {report && (
        <Modal
          wide
          title={t('settings.diagnostics')}
          onClose={() => setReport(null)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setReport(null)}>
                {t('common.close')}
              </Button>
              <Button
                variant="primary"
                icon="copy"
                onClick={() =>
                  navigator.clipboard.writeText(report).then(
                    () => toast.ok(t('common.copied')),
                    (e) => toast.error(errorText(t, e)),
                  )
                }
              >
                {t('common.copy')}
              </Button>
            </>
          }
        >
          <p className="small muted" style={{ marginBottom: 10 }}>
            {t('settings.diagnosticsHint')}
          </p>
          <pre className="mono small selectable card" style={{ whiteSpace: 'pre-wrap', margin: 0, padding: 14 }}>
            {report}
          </pre>
        </Modal>
      )}
      {confirmReset && (
        <ConfirmDialog
          title={t('settings.reset')}
          danger
          confirmLabel={t('settings.reset')}
          onCancel={() => setConfirmReset(false)}
          onConfirm={async () => {
            try {
              await reset();
              toast.ok(t('settings.resetDone'));
            } catch (e) {
              toast.error(errorText(t, e));
            }
            setConfirmReset(false);
          }}
        >
          <p className="muted">{t('settings.resetBody')}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}
