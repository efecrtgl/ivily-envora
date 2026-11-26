import { useEffect, useMemo, useState } from 'react';
import { useNav } from '../../app/navigation.js';
import { useSettings } from '../../app/settings.jsx';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api, inTauri } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Badge, Button, Callout, Card, Empty, Loading, PageHeader, SearchInput, Segmented, Toggle } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';

export const SEVERITY_TONE = { error: 'danger', warning: 'warn', info: 'info' };
const CATEGORY_PAGE = { path: 'path', variables: 'variables', tools: 'variables', security: 'variables', system: 'variables', process: 'settings', profiles: 'profiles' };

/** Kural kimliğini i18n anahtarına çevirir (noktalar iç içe anahtarlarla çakışmasın). */
export const ruleKey = (id) => `analysis.rule.${id.replace(/\./g, '_')}`;

export function filterFindings(findings, { severity, category, search }) {
  const q = (search || '').trim().toLowerCase();
  return findings.filter(
    (f) =>
      (severity === 'all' || f.severity === severity) &&
      (category === 'all' || f.category === category) &&
      (!q || f.target.toLowerCase().includes(q) || f.rule.toLowerCase().includes(q)),
  );
}

function RulesCard() {
  const t = useT();
  const toast = useToast();
  const { settings, set } = useSettings();
  const [rules, setRules] = useState(null);
  useEffect(() => {
    if (inTauri()) api.analysisRules().then(setRules).catch(() => setRules([]));
  }, [settings.analysis]);
  if (!rules) return null;
  const disabled = settings.analysis.disabledRules;
  const toggle = (id, on) =>
    set('analysis.disabledRules', on ? disabled.filter((d) => d !== id) : [...disabled, id]).catch((e) => toast.error(errorText(t, e)));
  const severity = (id, value, def) => {
    const next = { ...settings.analysis.severity };
    if (value === def) delete next[id];
    else next[id] = value;
    set('analysis.severity', next).catch((e) => toast.error(errorText(t, e)));
  };
  return (
    <Card title={t('analysis.rulesTitle')} icon="settings" desc={t('analysis.rulesDesc')}>
      <div className="list">
        {rules.map((r) => (
          <div key={r.id} className="list-item" style={{ paddingLeft: 0, paddingRight: 0 }}>
            <Toggle label={t(`${ruleKey(r.id)}.title`)} checked={r.enabled} onChange={(on) => toggle(r.id, on)} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="small">{t(`${ruleKey(r.id)}.title`)}</div>
              <div className="xsmall faint mono">{r.id}</div>
            </div>
            <select className="select" style={{ width: 120 }} value={r.severity} onChange={(e) => severity(r.id, e.target.value, r.defaultSeverity)} disabled={!r.enabled} aria-label={t('analysis.severity')}>
              {['info', 'warning', 'error'].map((s) => (
                <option key={s} value={s}>
                  {t(`severity.${s}`)}
                </option>
              ))}
            </select>
          </div>
        ))}
      </div>
    </Card>
  );
}

export default function Analysis() {
  const t = useT();
  const lang = useLang();
  const { navigate } = useNav();
  const [state, setState] = useState({ loading: true, data: null, error: null });
  const [severity, setSeverity] = useState('all');
  const [category, setCategory] = useState('all');
  const [search, setSearch] = useState('');
  const [showRules, setShowRules] = useState(false);

  const run = () => {
    setState((s) => ({ ...s, loading: true, error: null }));
    api.analysisRun().then(
      (data) => setState({ loading: false, data, error: null }),
      (error) => setState((s) => ({ loading: false, data: s.data, error })),
    );
  };

  useEffect(() => {
    if (!inTauri()) {
      setState({ loading: false, data: null, error: null });
      return;
    }
    // Son sonuç varsa onu göster; yoksa bir kez çalıştır.
    api.analysisLast().then((last) => (last ? setState({ loading: false, data: last, error: null }) : run()), run);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const d = state.data;
  const categories = useMemo(() => [...new Set((d?.findings || []).map((f) => f.category))].sort(), [d]);
  const rows = useMemo(() => filterFindings(d?.findings || [], { severity, category, search }), [d, severity, category, search]);

  return (
    <div className="page">
      <PageHeader
        title={t('analysis.title')}
        subtitle={t('analysis.subtitle')}
        actions={
          <>
            <Button icon="settings" variant="ghost" onClick={() => setShowRules(!showRules)}>
              {t('analysis.rulesTitle')}
            </Button>
            <Button variant="primary" icon="refresh" onClick={run} disabled={state.loading}>
              {t('analysis.run')}
            </Button>
          </>
        }
      />
      <div className="stack">
        {showRules && <RulesCard />}
        {state.error && <Callout tone="danger">{errorText(t, state.error)}</Callout>}
        {state.loading && !d && <Loading label={t('analysis.running')} />}
        {d && (
          <>
            <div className="grid cols-4">
              {['error', 'warning', 'info'].map((s) => (
                <Card key={s}>
                  <div className="stat-value" style={{ color: d.counts[s] ? `var(--${s === 'error' ? 'danger' : s === 'warning' ? 'warn' : 'info'})` : undefined }}>
                    {d.counts[s] || 0}
                  </div>
                  <div className="stat-label">{t(`analysis.count.${s}`)}</div>
                </Card>
              ))}
              <Card>
                <div className="stat-value">{d.rulesRun}</div>
                <div className="stat-label">{t('analysis.rulesRun')}</div>
                <div className="xsmall faint" style={{ marginTop: 6 }}>
                  {formatDate(lang, d.generatedAt)} · {d.durationMs} ms
                </div>
              </Card>
            </div>
            <Card flush>
              <div className="toolbar">
                <SearchInput value={search} onChange={setSearch} placeholder={t('common.search')} />
                <Segmented
                  label={t('analysis.severity')}
                  value={severity}
                  onChange={setSeverity}
                  options={['all', 'error', 'warning', 'info'].map((s) => ({ value: s, label: s === 'all' ? t('common.all') : t(`severity.${s}`) }))}
                />
                <select className="select" style={{ width: 170 }} value={category} onChange={(e) => setCategory(e.target.value)} aria-label={t('vars.category')}>
                  <option value="all">{t('vars.allCategories')}</option>
                  {categories.map((c) => (
                    <option key={c} value={c}>
                      {t(`analysis.category.${c}`)}
                    </option>
                  ))}
                </select>
              </div>
              {d.findings.length === 0 ? (
                <Empty icon="check" title={t('analysis.cleanTitle')}>
                  {t('analysis.cleanBody')}
                </Empty>
              ) : rows.length === 0 ? (
                <Empty icon="search" title={t('vars.noResults')} />
              ) : (
                <div className="list">
                  {rows.map((f, i) => (
                    <div key={`${f.rule}:${f.target}:${f.scope}:${i}`} className="list-item" style={{ alignItems: 'flex-start' }}>
                      <Badge tone={SEVERITY_TONE[f.severity]}>{t(`severity.${f.severity}`)}</Badge>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div className="row">
                          <strong className="small">{t(`${ruleKey(f.rule)}.title`)}</strong>
                          {f.scope && <ScopeBadge scope={f.scope} />}
                        </div>
                        <div className="mono small break selectable" style={{ margin: '2px 0' }}>
                          {f.target}
                        </div>
                        <div className="small muted">{t(`${ruleKey(f.rule)}.body`, f.params)}</div>
                      </div>
                      <Button size="sm" variant="ghost" icon="chevronRight" onClick={() => navigate(CATEGORY_PAGE[f.category] || 'variables')}>
                        {t('analysis.goTo')}
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </Card>
            <p className="xsmall faint">{t('analysis.note')}</p>
          </>
        )}
      </div>
    </div>
  );
}
