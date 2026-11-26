import { useEffect, useMemo, useState } from 'react';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api, inTauri } from '../../lib/api.js';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Checkbox, Empty, Loading, PageHeader } from '../../components/ui/primitives.jsx';
import VariableEditor from '../../components/env/VariableEditor.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';

/** Java bulunduysa JAVA_HOME önerisi: java.exe'nin bulunduğu bin dizininin üst dizini. */
export function javaHomeSuggestion(tools) {
  const java = tools.find((t) => (t.id === 'javac' || t.id === 'java') && t.found && !t.appAlias);
  const path = java?.locations[0]?.path;
  if (!path) return null;
  const parts = path.split('\\');
  if (parts.length < 3 || parts[parts.length - 2].toLowerCase() !== 'bin') return null;
  return parts.slice(0, -2).join('\\');
}

export function groupTools(tools, showMissing) {
  const groups = {};
  for (const t of tools) {
    if (!t.found && !showMissing) continue;
    (groups[t.category] ||= []).push(t);
  }
  return groups;
}

function ToolRow({ t: tool }) {
  const t = useT();
  return (
    <div className="list-item" style={{ alignItems: 'flex-start' }}>
      <span className={`dot ${tool.found ? (tool.appAlias ? 'warn' : 'ok') : ''}`} style={{ marginTop: 6 }} />
      <div style={{ width: 200, flex: 'none' }}>
        <strong className="small">{tool.name}</strong>
        <div className="mono xsmall faint">{tool.id}</div>
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        {!tool.found ? (
          <span className="small faint">{t('tools.notFound')}</span>
        ) : (
          <div className="stack tight" style={{ gap: 4 }}>
            {tool.locations.map((l, i) => (
              <div key={l.path} className="row" style={{ flexWrap: 'nowrap' }}>
                {tool.locations.length > 1 && (i === 0 ? <Badge tone="ok">{t('pathmgr.wins')}</Badge> : <Badge>{t('pathmgr.shadowed')}</Badge>)}
                <span className="mono xsmall truncate selectable" title={l.path}>
                  {l.path}
                </span>
                {l.appAlias && (
                  <Badge tone="warn" title={t('pathmgr.appAliasHelp')}>
                    {t('pathmgr.appAlias')}
                  </Badge>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
      <div style={{ width: 230, flex: 'none', textAlign: 'right' }}>
        {tool.version ? (
          <>
            <div className="mono small truncate" title={tool.version}>
              {tool.version}
            </div>
            <div className="xsmall faint">{t(`tools.source.${tool.versionSource}`)}</div>
          </>
        ) : (
          tool.found && <span className="xsmall faint">{tool.appAlias ? t('tools.aliasNotRun') : t('tools.noVersion')}</span>
        )}
      </div>
    </div>
  );
}

export default function Tools() {
  const t = useT();
  const lang = useLang();
  const [state, setState] = useState({ loading: true, data: null, error: null });
  const [showMissing, setShowMissing] = useState(false);
  const [javaHomeSet, setJavaHomeSet] = useState(true);
  const [editor, setEditor] = useState(null);
  const [flow, setFlow] = useState(null);

  const run = () => {
    setState((s) => ({ ...s, loading: true, error: null }));
    api.discoveryRun().then(
      (data) => setState({ loading: false, data, error: null }),
      (error) => setState((s) => ({ loading: false, data: s.data, error })),
    );
  };

  useEffect(() => {
    if (!inTauri()) return setState({ loading: false, data: null, error: null });
    api.discoveryLast().then((d) => (d ? setState({ loading: false, data: d, error: null }) : setState({ loading: false, data: null, error: null })));
    api.envDetail('JAVA_HOME').then(
      () => setJavaHomeSet(true),
      () => setJavaHomeSet(false),
    );
  }, []);

  const d = state.data;
  const groups = useMemo(() => groupTools(d?.tools || [], showMissing), [d, showMissing]);
  const javaHome = d && !javaHomeSet ? javaHomeSuggestion(d.tools) : null;
  const found = d ? d.tools.filter((x) => x.found).length : 0;

  return (
    <div className="page">
      <PageHeader
        title={t('tools.title')}
        subtitle={t('tools.subtitle')}
        actions={
          <Button variant="primary" icon="search" onClick={run} disabled={state.loading && !!d}>
            {d ? t('tools.rescan') : t('tools.scan')}
          </Button>
        }
      />
      <div className="stack">
        <Callout tone="info">{t('tools.safety')}</Callout>
        {state.error && <Callout tone="danger">{errorText(t, state.error)}</Callout>}
        {state.loading && <Loading label={t('tools.scanning')} />}
        {!state.loading && !d && (
          <Card>
            <Empty icon="tools" title={t('tools.emptyTitle')} action={<Button variant="primary" icon="search" onClick={run}>{t('tools.scan')}</Button>}>
              {t('tools.emptyBody')}
            </Empty>
          </Card>
        )}
        {d && (
          <>
            <div className="row">
              <Badge tone="ok">{t('tools.foundCount', { count: found, total: d.tools.length })}</Badge>
              <span className="xsmall faint">
                {formatDate(lang, d.generatedAt)} · {d.durationMs} ms · {t('tools.probed', { count: d.probed })}
              </span>
              <span className="spacer" />
              <Checkbox checked={showMissing} onChange={setShowMissing}>
                {t('tools.showMissing')}
              </Checkbox>
            </div>
            {javaHome && (
              <Callout tone="info" icon="sparkle" title={t('tools.suggestion')}>
                <div className="row">
                  <span>{t('tools.javaHomeSuggestion', { dir: javaHome })}</span>
                  <Button size="sm" variant="primary" onClick={() => setEditor({ mode: 'create', initial: { name: 'JAVA_HOME', value: javaHome } })}>
                    {t('tools.review')}
                  </Button>
                </div>
              </Callout>
            )}
            {Object.entries(groups).map(([cat, list]) => (
              <Card key={cat} flush title={t(`tools.category.${cat}`)} icon="tools">
                <div className="list">
                  {list.map((tool) => (
                    <ToolRow key={tool.id} t={tool} />
                  ))}
                </div>
              </Card>
            ))}
            <p className="xsmall faint">
              <Icon name="info" size={12} /> {t('tools.note')}
            </p>
          </>
        )}
      </div>
      {editor && (
        <VariableEditor
          mode={editor.mode}
          initial={editor.initial}
          onClose={() => setEditor(null)}
          onPlan={(plan) => {
            setEditor(null);
            setFlow(plan);
          }}
        />
      )}
      {flow && <ChangeFlow plan={flow} onClose={() => setFlow(null)} onDone={() => setJavaHomeSet(true)} />}
    </div>
  );
}
