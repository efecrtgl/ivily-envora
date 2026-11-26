import { useEffect, useMemo, useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import * as D from '../../lib/pathDraft.js';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Checkbox, Empty, Loading, Meter, PageHeader, SearchInput, Segmented, Tabs } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';

const LEGACY_LEN = 2047;
const MAX_LEN = 32767;

const FLAG_TONE = {
  empty: 'warn',
  missing: 'danger',
  notDirectory: 'danger',
  relative: 'warn',
  duplicate: 'warn',
  duplicateOtherScope: 'info',
  unresolvedReference: 'danger',
  quoted: 'warn',
  whitespace: 'warn',
  network: 'info',
};

function Flags({ flags }) {
  const t = useT();
  return flags.map((f) => (
    <Badge key={f} tone={FLAG_TONE[f]} title={t(`pathmgr.flagHelp.${f}`)}>
      {t(`pathmgr.flag.${f}`)}
    </Badge>
  ));
}

function Conflicts() {
  const t = useT();
  const [state, setState] = useState({ loading: false, data: null, error: null });
  const run = () => {
    setState({ loading: true, data: null, error: null });
    api.pathConflicts().then(
      (data) => setState({ loading: false, data, error: null }),
      (error) => setState({ loading: false, data: null, error }),
    );
  };
  return (
    <Card
      title={t('pathmgr.conflictsTitle')}
      icon="compare"
      desc={t('pathmgr.conflictsDesc')}
      actions={
        <Button size="sm" icon="search" onClick={run} disabled={state.loading}>
          {state.data ? t('common.refresh') : t('pathmgr.analyze')}
        </Button>
      }
    >
      {state.loading && <Loading />}
      {state.error && <Callout tone="danger">{errorText(t, state.error)}</Callout>}
      {state.data && state.data.length === 0 && <p className="small faint">{t('pathmgr.noConflicts')}</p>}
      {state.data && state.data.length > 0 && (
        <div className="list">
          {state.data.map((c) => (
            <div key={c.name} className="list-item" style={{ alignItems: 'flex-start', paddingLeft: 0, paddingRight: 0 }}>
              <span className="mono small" style={{ width: 160, flex: 'none', fontWeight: 600 }}>
                {c.name}
              </span>
              <div className="stack tight" style={{ flex: 1, minWidth: 0, gap: 4 }}>
                {c.locations.map((l, i) => (
                  <div key={l.dir} className="row" style={{ flexWrap: 'nowrap' }}>
                    {i === 0 ? <Badge tone="ok">{t('pathmgr.wins')}</Badge> : <Badge>{t('pathmgr.shadowed')}</Badge>}
                    <ScopeBadge scope={l.scope} />
                    <span className="mono xsmall truncate" title={l.dir}>
                      {l.dir}
                    </span>
                    {l.appAlias && <Badge tone="warn" title={t('pathmgr.appAliasHelp')}>{t('pathmgr.appAlias')}</Badge>}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
}

function ScopePath({ scope }) {
  const t = useT();
  const { loading, error, data: report, reload } = useAsync(() => api.pathAnalyze(scope), [scope]);
  const [draft, setDraft] = useState([]);
  const [selected, setSelected] = useState(new Set());
  const [search, setSearch] = useState('');
  const [newEntry, setNewEntry] = useState('');
  const [position, setPosition] = useState('end');
  const [editing, setEditing] = useState(null); // { id, raw }
  const [flow, setFlow] = useState(null);

  useEffect(() => {
    if (report) {
      setDraft(D.draftFromReport(report));
      setSelected(new Set());
    }
  }, [report]);

  const dirty = report ? D.isDirty(draft, report) : false;
  const readOnly = report?.readOnly;
  const length = useMemo(() => D.draftValue(draft).length, [draft]);
  const q = search.trim().toLowerCase();
  const visible = draft.filter((d) => !q || d.raw.toLowerCase().includes(q) || d.entry?.expanded.toLowerCase().includes(q));

  if (loading && !report) return <Loading label={t('common.loading')} />;
  if (error) return <Callout tone="danger">{errorText(t, error)}</Callout>;
  if (!report) return null;

  const select = (ids) => setSelected(new Set(ids));
  const toggle = (id) => {
    const s = new Set(selected);
    s.has(id) ? s.delete(id) : s.add(id);
    setSelected(s);
  };
  const add = () => {
    if (!newEntry.trim() || newEntry.includes(';')) return;
    setDraft(D.addEntry(draft, newEntry, position));
    setNewEntry('');
  };
  const review = () =>
    setFlow({
      scope,
      ops: [{ op: 'set', name: report.name || 'Path', value: D.draftValue(draft), kind: report.kind || 'expand' }],
    });

  const counts = report.counts;
  const tone = length > LEGACY_LEN ? 'warn' : undefined;

  return (
    <div className="stack">
      <div className="grid cols-4">
        <Card>
          <div className="stat-value">{report.entries.length}</div>
          <div className="stat-label">{t('pathmgr.entries')}</div>
        </Card>
        <Card>
          <div className="stat-value" style={{ color: (counts.missing || 0) + (counts.notDirectory || 0) ? 'var(--danger)' : undefined }}>
            {(counts.missing || 0) + (counts.notDirectory || 0)}
          </div>
          <div className="stat-label">{t('pathmgr.missingCount')}</div>
        </Card>
        <Card>
          <div className="stat-value" style={{ color: counts.duplicate ? 'var(--warn)' : undefined }}>
            {counts.duplicate || 0}
          </div>
          <div className="stat-label">{t('pathmgr.duplicateCount')}</div>
        </Card>
        <Card>
          <div className="row between">
            <span className="stat-label">{t('pathmgr.length')}</span>
            <span className="small mono">
              {length} / {LEGACY_LEN}
            </span>
          </div>
          <div style={{ margin: '10px 0 6px' }}>
            <Meter value={length} max={LEGACY_LEN} tone={tone} />
          </div>
          <div className="xsmall faint">{length > LEGACY_LEN ? t('pathmgr.lengthWarn') : t('pathmgr.lengthOk', { max: MAX_LEN })}</div>
        </Card>
      </div>

      {readOnly && <Callout tone="info">{t('pathmgr.processReadOnly')}</Callout>}
      {!report.name && !readOnly && <Callout tone="info">{t('pathmgr.noPathVar')}</Callout>}

      <Card flush>
        <div className="toolbar">
          <SearchInput value={search} onChange={setSearch} placeholder={t('pathmgr.search')} />
          {!readOnly && (
            <>
              <Button size="sm" variant="ghost" onClick={() => select(D.idsWithFlag(draft, 'duplicate'))} disabled={!counts.duplicate}>
                {t('pathmgr.selectDuplicates')}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => select([...D.idsWithFlag(draft, 'missing'), ...D.idsWithFlag(draft, 'notDirectory')])} disabled={!counts.missing && !counts.notDirectory}>
                {t('pathmgr.selectMissing')}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => select(D.idsWithFlag(draft, 'empty'))} disabled={!counts.empty}>
                {t('pathmgr.selectEmpty')}
              </Button>
              <Button
                size="sm"
                variant="danger"
                icon="trash"
                disabled={!selected.size}
                onClick={() => {
                  setDraft(D.removeIds(draft, [...selected]));
                  setSelected(new Set());
                }}
              >
                {t('pathmgr.removeSelected', { count: selected.size })}
              </Button>
            </>
          )}
        </div>
        {visible.length === 0 ? (
          <Empty icon="path" title={draft.length ? t('vars.noResults') : t('pathmgr.empty')} />
        ) : (
          <div className="list">
            {visible.map((d) => {
              const i = draft.indexOf(d);
              const e = d.entry;
              return (
                <div key={d.id} className={`list-item ${selected.has(d.id) ? 'selected' : ''}`} style={{ alignItems: 'flex-start' }}>
                  {!readOnly && <input type="checkbox" checked={selected.has(d.id)} onChange={() => toggle(d.id)} aria-label={d.raw || t('pathmgr.emptyEntry')} style={{ marginTop: 4 }} />}
                  <span className="xsmall faint mono" style={{ width: 24, textAlign: 'right', marginTop: 3 }}>
                    {i + 1}
                  </span>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    {editing?.id === d.id ? (
                      <div className="row" style={{ flexWrap: 'nowrap' }}>
                        <input
                          className="input mono"
                          autoFocus
                          value={editing.raw}
                          onChange={(ev) => setEditing({ ...editing, raw: ev.target.value })}
                          onKeyDown={(ev) => {
                            if (ev.key === 'Enter' && !editing.raw.includes(';')) {
                              setDraft(D.editEntry(draft, d.id, editing.raw.trim()));
                              setEditing(null);
                            }
                            if (ev.key === 'Escape') setEditing(null);
                          }}
                        />
                        <Button size="sm" variant="primary" disabled={editing.raw.includes(';')} onClick={() => { setDraft(D.editEntry(draft, d.id, editing.raw.trim())); setEditing(null); }}>
                          {t('common.save')}
                        </Button>
                      </div>
                    ) : (
                      <>
                        <div className="mono small break selectable">{d.raw || <span className="faint">{t('pathmgr.emptyEntry')}</span>}</div>
                        {e && !d.edited && e.expanded !== d.raw.trim() && <div className="mono xsmall faint break">→ {e.expanded}</div>}
                        <div className="row" style={{ marginTop: 4 }}>
                          {d.edited ? <Badge tone="info">{t('pathmgr.changed')}</Badge> : <Flags flags={e?.flags || []} />}
                        </div>
                      </>
                    )}
                  </div>
                  {!readOnly && editing?.id !== d.id && (
                    <div className="row" style={{ flexWrap: 'nowrap', flex: 'none' }}>
                      <Button size="sm" variant="ghost" iconOnly icon="arrowUp" title={t('envset.moveUp')} disabled={i === 0} onClick={() => setDraft(D.move(draft, d.id, -1))} />
                      <Button size="sm" variant="ghost" iconOnly icon="arrowDown" title={t('envset.moveDown')} disabled={i === draft.length - 1} onClick={() => setDraft(D.move(draft, d.id, 1))} />
                      <Button size="sm" variant="ghost" iconOnly icon="edit" title={t('common.edit')} onClick={() => setEditing({ id: d.id, raw: d.raw })} />
                      <Button size="sm" variant="ghost" iconOnly icon="x" title={t('common.delete')} onClick={() => setDraft(D.removeIds(draft, [d.id]))} />
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
        {!readOnly && (
          <div className="toolbar" style={{ padding: '14px 20px', margin: 0, borderTop: '1px solid var(--border)' }}>
            <input
              className="input mono"
              style={{ flex: 1, minWidth: 260 }}
              value={newEntry}
              onChange={(e) => setNewEntry(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && add()}
              placeholder={t('pathmgr.newPlaceholder')}
              aria-invalid={newEntry.includes(';')}
              spellCheck={false}
            />
            <Segmented
              label={t('pathmgr.position')}
              value={position}
              onChange={setPosition}
              options={[
                { value: 'start', label: t('envset.prepend') },
                { value: 'end', label: t('envset.append') },
              ]}
            />
            <Button icon="plus" onClick={add} disabled={!newEntry.trim() || newEntry.includes(';')}>
              {t('common.add')}
            </Button>
            {newEntry.trim() && D.exists(draft, newEntry) && <span className="field-error">{t('pathmgr.alreadyExists')}</span>}
          </div>
        )}
      </Card>

      {!readOnly && (
        <div className="sticky-actions">
          <span className="small muted">{dirty ? t('pathmgr.unsaved') : t('pathmgr.noChangesYet')}</span>
          <span className="spacer" />
          <Button variant="ghost" icon="undo" disabled={!dirty} onClick={() => setDraft(D.draftFromReport(report))}>
            {t('pathmgr.discard')}
          </Button>
          <Button variant="primary" icon="check" disabled={!dirty} onClick={review}>
            {t('editor.review')}
          </Button>
        </div>
      )}

      {flow && <ChangeFlow plan={flow} title={t('pathmgr.applyTitle', { scope: t(`scope.${scope}`) })} onClose={() => setFlow(null)} onDone={reload} />}
    </div>
  );
}

export default function PathManager() {
  const t = useT();
  const [scope, setScope] = useState('user');
  return (
    <div className="page">
      <PageHeader title={t('pathmgr.title')} subtitle={t('pathmgr.subtitle')} />
      <Tabs
        value={scope}
        onChange={setScope}
        tabs={[
          { value: 'user', label: t('scope.user') },
          { value: 'machine', label: t('scope.machine') },
          { value: 'process', label: t('scope.process') },
        ]}
      />
      <div className="stack">
        <ScopePath key={scope} scope={scope} />
        <Conflicts />
        <p className="xsmall faint">
          <Icon name="info" size={12} /> {t('pathmgr.orderNote')}
        </p>
      </div>
    </div>
  );
}
