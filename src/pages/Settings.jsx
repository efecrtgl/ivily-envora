import { useCallback, useEffect, useState } from 'react';
import { save as saveDialog } from '@tauri-apps/plugin-dialog';
import { useT, LANGUAGES, errorText, useLang } from '../i18n/index.jsx';
import { useSettings } from '../app/settings.jsx';
import { PAGES } from '../app/nav.js';
import { api, inTauri } from '../lib/api.js';
import { formatBytes } from '../lib/format.js';
import { useToast } from '../components/ui/Toast.jsx';
import { ConfirmDialog } from '../components/ui/Modal.jsx';
import { Badge, Button, Callout, Card, PageHeader, Segmented, Toggle } from '../components/ui/primitives.jsx';

const START_PAGES = ['dashboard', 'variables', 'path', 'profiles', 'projects'];

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

  return (
    <dl className="kv">
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
        <Button size="sm" variant="ghost" icon="refresh" onClick={load} disabled={!inTauri()}>
          {t('common.refresh')}
        </Button>
      </dd>
    </dl>
  );
}

export default function Settings() {
  const t = useT();
  const lang = useLang();
  const toast = useToast();
  const { settings: s, meta, set, reset } = useSettings();
  const [info, setInfo] = useState(null);
  const [confirmReset, setConfirmReset] = useState(false);

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

        <Card title={t('settings.activity')} icon="activity">
          <SettingRow label={t('settings.activityEnabled')} hint={t('settings.activityEnabledHint')}>
            <Toggle label={t('settings.activityEnabled')} checked={s.activity.enabled} onChange={change('activity.enabled')} />
          </SettingRow>
          <SettingRow label={t('settings.retention')} hint={t('settings.retentionHint')}>
            <NumberSetting value={s.activity.retentionDays} min={1} max={3650} suffix={t('settings.days')} onCommit={change('activity.retentionDays')} />
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
            <Button size="sm" variant="danger" icon="undo" onClick={() => setConfirmReset(true)}>
              {t('settings.reset')}
            </Button>
          </div>
        </Card>
      </div>

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
