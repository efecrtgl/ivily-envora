import { useEffect, useState } from 'react';
import { useNav } from '../app/navigation.js';
import { useSettings } from '../app/settings.jsx';
import { useT, errorText } from '../i18n/index.jsx';
import { api, inTauri } from '../lib/api.js';
import { useAsync } from '../lib/useAsync.js';
import Icon from '../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Loading, Meter, PageHeader } from '../components/ui/primitives.jsx';
import ScopeBadge from '../components/env/ScopeBadge.jsx';
import { EntryLine } from './activity/Activity.jsx';
import { SEVERITY_TONE, ruleKey } from './analysis/Analysis.jsx';

const LEGACY_LEN = 2047;

function Section({ title, icon, page, children, actions }) {
  const t = useT();
  const { navigate } = useNav();
  return (
    <Card
      title={title}
      icon={icon}
      actions={
        <>
          {actions}
          {page && (
            <Button size="sm" variant="ghost" icon="chevronRight" onClick={() => navigate(page)}>
              {t('dashboard.open')}
            </Button>
          )}
        </>
      }
    >
      {children}
    </Card>
  );
}

function SectionBody({ state, children }) {
  const t = useT();
  if (state.error) return <p className="small" style={{ color: 'var(--danger)' }}>{errorText(t, state.error)}</p>;
  if (!state.data) return <Loading />;
  return children(state.data);
}

function Overview() {
  const t = useT();
  const s = useAsync(() => api.envOverview(), []);
  return (
    <Section title={t('dashboard.overview')} icon="variable" page="variables">
      <SectionBody state={s}>
        {(o) => (
          <>
            <div className="grid cols-3">
              {o.scopes.map((sc) => (
                <div key={sc.scope}>
                  <div className="stat-value">{sc.total}</div>
                  <div className="row">
                    <ScopeBadge scope={sc.scope} />
                    {sc.sensitive > 0 && (
                      <Badge icon="lock" title={t('vars.sensitive')}>
                        {sc.sensitive}
                      </Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
            <p className="xsmall faint" style={{ marginTop: 12 }}>
              {o.canWriteMachine ? t('dashboard.adminYes') : t('dashboard.adminNo')}
              {o.shadowed.length > 0 && ` · ${t('dashboard.shadowed', { count: o.shadowed.length })}`}
            </p>
          </>
        )}
      </SectionBody>
    </Section>
  );
}

function PathHealth() {
  const t = useT();
  const s = useAsync(() => Promise.all([api.pathAnalyze('user'), api.pathAnalyze('machine')]), []);
  return (
    <Section title={t('dashboard.pathHealth')} icon="path" page="path">
      <SectionBody state={s}>
        {(reports) => (
          <div className="stack tight">
            {reports.map((r) => {
              const problems = (r.counts.missing || 0) + (r.counts.notDirectory || 0) + (r.counts.duplicate || 0) + (r.counts.unresolvedReference || 0) + (r.counts.relative || 0);
              return (
                <div key={r.scope}>
                  <div className="row between">
                    <span className="row">
                      <ScopeBadge scope={r.scope} />
                      <span className="small">{t('dashboard.entries', { count: r.entries.length })}</span>
                    </span>
                    {problems ? <Badge tone="warn">{t('dashboard.problems', { count: problems })}</Badge> : <Badge tone="ok">{t('dashboard.healthy')}</Badge>}
                  </div>
                  <div style={{ marginTop: 6 }}>
                    <Meter value={r.length} max={LEGACY_LEN} tone={r.length > LEGACY_LEN ? 'warn' : undefined} />
                  </div>
                  <div className="xsmall faint" style={{ marginTop: 2 }}>
                    {r.length} / {LEGACY_LEN}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </SectionBody>
    </Section>
  );
}

function AnalysisSummary() {
  const t = useT();
  // Analiz ucuzdur; önceki sonuç yoksa pano açılışında bir kez çalıştırılır (sürekli tarama yok).
  const s = useAsync(async () => (await api.analysisLast()) || (await api.analysisRun()), []);
  return (
    <Section title={t('dashboard.analysis')} icon="analysis" page="analysis" actions={<Button size="sm" variant="ghost" icon="refresh" title={t('analysis.run')} iconOnly onClick={() => api.analysisRun().then(() => s.reload())} />}>
      <SectionBody state={s}>
        {(r) => (
          <>
            <div className="row" style={{ marginBottom: 10 }}>
              {['error', 'warning', 'info'].map((k) => (
                <Badge key={k} tone={SEVERITY_TONE[k]}>
                  {r.counts[k] || 0} {t(`severity.${k}`)}
                </Badge>
              ))}
            </div>
            {r.findings.length === 0 ? (
              <p className="small muted">{t('analysis.cleanTitle')}</p>
            ) : (
              <div className="stack tight">
                {r.findings.slice(0, 4).map((f, i) => (
                  <div key={i} className="row small" style={{ flexWrap: 'nowrap' }}>
                    <span className={`dot ${f.severity === 'error' ? 'danger' : f.severity === 'warning' ? 'warn' : ''}`} />
                    <span className="truncate">{t(`${ruleKey(f.rule)}.title`)}</span>
                    <span className="mono xsmall faint truncate">{f.target}</span>
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </SectionBody>
    </Section>
  );
}

function SecurityStatus() {
  const t = useT();
  const s = useAsync(() => Promise.all([api.envOverview(), api.sensitivityList()]), []);
  const { settings } = useSettings();
  return (
    <Section title={t('dashboard.security')} icon="shield" page="settings">
      <SectionBody state={s}>
        {([o, overrides]) => {
          const user = o.scopes.find((x) => x.scope === 'user')?.sensitive || 0;
          const machine = o.scopes.find((x) => x.scope === 'machine')?.sensitive || 0;
          return (
            <div className="stack tight small">
              <div className="row between">
                <span>{t('dashboard.secretsUser')}</span>
                <Badge tone={user ? 'info' : 'ok'}>{user}</Badge>
              </div>
              <div className="row between">
                <span>{t('dashboard.secretsMachine')}</span>
                <Badge tone={machine ? 'warn' : 'ok'}>{machine}</Badge>
              </div>
              <div className="row between">
                <span>{t('settings.overrides')}</span>
                <Badge>{overrides.length}</Badge>
              </div>
              <div className="row between">
                <span>{t('settings.clipboardClear')}</span>
                <Badge>{settings.security.clipboardClearSeconds ? `${settings.security.clipboardClearSeconds} s` : t('common.no')}</Badge>
              </div>
            </div>
          );
        }}
      </SectionBody>
    </Section>
  );
}

function ProfilesSummary() {
  const t = useT();
  const { navigate } = useNav();
  const s = useAsync(() => api.profilesList(false), []);
  return (
    <Section title={t('nav.profiles')} icon="profile" page="profiles">
      <SectionBody state={s}>
        {(list) =>
          list.length === 0 ? (
            <p className="small muted">{t('profiles.emptyTitle')}</p>
          ) : (
            <div className="list">
              {list.slice(0, 5).map((p) => (
                <div key={p.id} className="list-item clickable" style={{ paddingLeft: 0, paddingRight: 0 }} onClick={() => navigate('profiles', { id: p.id })}>
                  <span className={`dot ${p.activeScopes.length ? 'ok' : ''}`} />
                  <span className="small truncate" style={{ flex: 1 }}>
                    {p.name}
                  </span>
                  {p.activeScopes.map((sc) => (
                    <ScopeBadge key={sc} scope={sc} />
                  ))}
                </div>
              ))}
            </div>
          )
        }
      </SectionBody>
    </Section>
  );
}

function ProjectsSummary() {
  const t = useT();
  const { navigate } = useNav();
  const s = useAsync(() => api.projectsList(), []);
  return (
    <Section title={t('nav.projects')} icon="project" page="projects">
      <SectionBody state={s}>
        {(list) =>
          list.length === 0 ? (
            <p className="small muted">{t('projects.emptyTitle')}</p>
          ) : (
            <div className="list">
              {[...list]
                .sort((a, b) => (b.updatedAt || '').localeCompare(a.updatedAt || ''))
                .slice(0, 5)
                .map((p) => (
                  <div key={p.id} className="list-item clickable" style={{ paddingLeft: 0, paddingRight: 0 }} onClick={() => navigate('projects', { id: p.id })}>
                    <Icon name="folder" size={14} />
                    <span className="small truncate" style={{ flex: 1 }}>
                      {p.name}
                    </span>
                    {!p.directoryExists && <Badge tone="danger">{t('projects.dirMissing')}</Badge>}
                  </div>
                ))}
            </div>
          )
        }
      </SectionBody>
    </Section>
  );
}

function ToolsSummary() {
  const t = useT();
  const { navigate } = useNav();
  // Keşif sürüm sorguları çalıştırabileceğinden pano bunu kendiliğinden başlatmaz.
  const s = useAsync(() => api.discoveryLast(), []);
  return (
    <Section title={t('nav.tools')} icon="tools" page="tools">
      {s.loading ? (
        <Loading />
      ) : !s.data ? (
        <div className="row between">
          <span className="small muted">{t('tools.emptyBody')}</span>
          <Button size="sm" icon="search" onClick={() => navigate('tools')}>
            {t('tools.scan')}
          </Button>
        </div>
      ) : (
        <div className="row">
          {s.data.tools
            .filter((x) => x.found)
            .map((x) => (
              <Badge key={x.id} tone={x.appAlias ? 'warn' : undefined} title={x.version || ''}>
                {x.name}
                {x.version && ` ${x.version.split(' ').find((p) => /\d/.test(p)) || ''}`}
              </Badge>
            ))}
        </div>
      )}
    </Section>
  );
}

function RecentActivity() {
  const t = useT();
  const s = useAsync(() => api.activityList({ limit: 6 }), []);
  return (
    <Section title={t('dashboard.activity')} icon="activity" page="activity">
      <SectionBody state={s}>
        {(rows) =>
          rows.length === 0 ? (
            <p className="small muted">{t('activity.empty')}</p>
          ) : (
            <div className="list" style={{ margin: '0 -16px' }}>
              {rows.map((e) => (
                <EntryLine key={e.id} e={e} compact />
              ))}
            </div>
          )
        }
      </SectionBody>
    </Section>
  );
}

function QuickActions() {
  const t = useT();
  const { navigate } = useNav();
  const actions = [
    ['variable', 'variables', 'dashboard.qa.variables'],
    ['path', 'path', 'dashboard.qa.path'],
    ['profile', 'profiles', 'dashboard.qa.profiles'],
    ['project', 'projects', 'dashboard.qa.projects'],
    ['snapshot', 'snapshots', 'dashboard.qa.snapshot'],
    ['importExport', 'transfer', 'dashboard.qa.transfer'],
  ];
  return (
    <div className="row">
      {actions.map(([icon, page, key]) => (
        <Button key={page} icon={icon} onClick={() => navigate(page)}>
          {t(key)}
        </Button>
      ))}
    </div>
  );
}

export const SECTIONS = {
  overview: Overview,
  pathHealth: PathHealth,
  analysis: AnalysisSummary,
  security: SecurityStatus,
  profiles: ProfilesSummary,
  projects: ProjectsSummary,
  tools: ToolsSummary,
  activity: RecentActivity,
};

export default function Dashboard() {
  const t = useT();
  const { settings } = useSettings();
  const [ready, setReady] = useState(inTauri());
  useEffect(() => setReady(inTauri()), []);
  const order = settings.dashboard.sections.filter((s) => SECTIONS[s]);

  return (
    <div className="page">
      <PageHeader title={t('dashboard.title')} subtitle={t('dashboard.subtitle')} />
      <div className="stack">
        <QuickActions />
        {!ready ? (
          <Callout>{t('errors.not_in_tauri')}</Callout>
        ) : order.length === 0 ? (
          <Callout>{t('dashboard.noSections')}</Callout>
        ) : (
          <div className="grid cols-2">
            {order.map((id) => {
              const C = SECTIONS[id];
              return <C key={id} />;
            })}
          </div>
        )}
      </div>
    </div>
  );
}
