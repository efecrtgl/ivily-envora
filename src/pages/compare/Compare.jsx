import { useEffect, useMemo, useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api, inTauri } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { Badge, Button, Callout, Card, Checkbox, Empty, Loading, PageHeader, SearchInput, Tabs } from '../../components/ui/primitives.jsx';
import { PathDiff } from '../../components/env/ChangeFlow.jsx';
import SourcePicker from './SourcePicker.jsx';

const PRESETS = [
  { key: 'userMachine', left: { kind: 'scope', scope: 'machine' }, right: { kind: 'scope', scope: 'user' } },
  { key: 'processEffective', left: { kind: 'scope', scope: 'process' }, right: { kind: 'effective' } },
];

const STATUS_TONE = { added: 'ok', removed: 'danger', changed: 'warn', unchanged: undefined };

export function filterRows(rows, { status, search, hideUnchanged }) {
  const q = (search || '').trim().toLowerCase();
  return rows.filter((r) => {
    if (status && status !== 'all' && r.status !== status) return false;
    if (hideUnchanged && (!status || status === 'all') && r.status === 'unchanged') return false;
    if (!q) return true;
    return r.name.toLowerCase().includes(q) || (r.left || '').toLowerCase().includes(q) || (r.right || '').toLowerCase().includes(q);
  });
}

/** Kullanılabilir karşılaştırma kaynaklarını yükler (profiller, projeler, anlık görüntüler). */
export function useSourceLists() {
  return useAsync(async () => {
    if (!inTauri()) return { profiles: [], projects: [], snapshots: [] };
    const [profiles, projects, snapshots] = await Promise.all([
      api.profilesList(true),
      api.projectsList(),
      api.snapshotsList ? api.snapshotsList().catch(() => []) : [],
    ]);
    return { profiles, projects, snapshots };
  }, []);
}

function Value({ v, present, sensitive }) {
  const t = useT();
  if (!present) return <span className="faint">—</span>;
  if (sensitive) return <span className="mask faint">••••••••</span>;
  if (v === '') return <span className="faint">{t('vars.emptyValue')}</span>;
  return <span className="mono break selectable">{v}</span>;
}

export default function Compare({ params }) {
  const t = useT();
  const lists = useSourceLists();
  const [left, setLeft] = useState(params?.left || PRESETS[0].left);
  const [right, setRight] = useState(params?.right || PRESETS[0].right);
  const [state, setState] = useState({ loading: false, data: null, error: null });
  const [status, setStatus] = useState('all');
  const [search, setSearch] = useState('');
  const [hideUnchanged, setHideUnchanged] = useState(true);

  const run = (l = left, r = right) => {
    setState({ loading: true, data: null, error: null });
    api.compareSources(l, r).then(
      (data) => setState({ loading: false, data, error: null }),
      (error) => setState({ loading: false, data: null, error }),
    );
  };

  useEffect(() => {
    if (params?.left && params?.right) run(params.left, params.right);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const rows = useMemo(() => (state.data ? filterRows(state.data.rows, { status, search, hideUnchanged }) : []), [state.data, status, search, hideUnchanged]);
  const d = state.data;
  const pickerProps = { profiles: lists.data?.profiles, projects: lists.data?.projects, snapshots: lists.data?.snapshots };

  return (
    <div className="page">
      <PageHeader title={t('compare.title')} subtitle={t('compare.subtitle')} />
      <Card>
        <div className="compare-pickers">
          <div className="field">
            <span className="field-label">{t('compare.left')}</span>
            <SourcePicker value={left} onChange={setLeft} label={t('compare.left')} {...pickerProps} />
          </div>
          <Button
            variant="ghost"
            iconOnly
            icon="compare"
            title={t('compare.swap')}
            onClick={() => {
              setLeft(right);
              setRight(left);
            }}
          />
          <div className="field">
            <span className="field-label">{t('compare.right')}</span>
            <SourcePicker value={right} onChange={setRight} label={t('compare.right')} {...pickerProps} />
          </div>
          <Button variant="primary" icon="compare" onClick={() => run()} disabled={state.loading}>
            {t('compare.run')}
          </Button>
        </div>
        <div className="row" style={{ marginTop: 12 }}>
          <span className="xsmall faint">{t('compare.presets')}</span>
          {PRESETS.map((p) => (
            <Button
              key={p.key}
              size="sm"
              variant="ghost"
              onClick={() => {
                setLeft(p.left);
                setRight(p.right);
                run(p.left, p.right);
              }}
            >
              {t(`compare.preset.${p.key}`)}
            </Button>
          ))}
        </div>
      </Card>

      <div style={{ marginTop: 16 }}>
        {state.loading && <Loading label={t('compare.running')} />}
        {state.error && <Callout tone="danger">{errorText(t, state.error)}</Callout>}
        {!d && !state.loading && !state.error && (
          <Card>
            <Empty icon="compare" title={t('compare.emptyTitle')}>
              {t('compare.emptyBody')}
            </Empty>
          </Card>
        )}
        {d && (
          <Card flush>
            <div className="toolbar">
              <Badge tone="ok">{t('compare.added', { count: d.added })}</Badge>
              <Badge tone="danger">{t('compare.removed', { count: d.removed })}</Badge>
              <Badge tone="warn">{t('compare.changed', { count: d.changed })}</Badge>
              <Badge>{t('compare.unchanged', { count: d.unchanged })}</Badge>
              {d.sensitive > 0 && (
                <Badge icon="lock" title={t('compare.sensitiveHint')}>
                  {t('compare.sensitive', { count: d.sensitive })}
                </Badge>
              )}
              <span className="spacer" />
              <SearchInput value={search} onChange={setSearch} placeholder={t('common.search')} />
              <Checkbox checked={hideUnchanged} onChange={setHideUnchanged}>
                {t('compare.hideUnchanged')}
              </Checkbox>
            </div>
            <div style={{ padding: '0 20px' }}>
              <Tabs
                value={status}
                onChange={setStatus}
                tabs={['all', 'added', 'removed', 'changed', 'unchanged'].map((s) => ({ value: s, label: t(`compare.status.${s}`) }))}
              />
            </div>
            {rows.length === 0 ? (
              <Empty icon="check" title={d.added + d.removed + d.changed === 0 ? t('compare.identical') : t('vars.noResults')} />
            ) : (
              <div className="table-wrap" style={{ maxHeight: 'calc(100vh - 380px)' }}>
                <table className="table fixed top">
                  <colgroup>
                    <col style={{ width: '24%' }} />
                    <col style={{ width: 110 }} />
                    <col />
                    <col />
                  </colgroup>
                  <thead>
                    <tr>
                      <th>{t('common.name')}</th>
                      <th>{t('compare.statusCol')}</th>
                      <th>{t('compare.left')}</th>
                      <th>{t('compare.right')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {rows.map((r) => (
                      <tr key={r.name} className={`diff-row ${r.status === 'added' ? 'added' : r.status === 'removed' ? 'removed' : r.status === 'changed' ? 'changed' : ''}`}>
                        <td className="name" title={r.name}>
                          {r.name}
                          {r.kindChanged && <div className="xsmall faint" style={{ fontFamily: 'var(--font-sans)', fontWeight: 400 }}>{t('compare.kindChanged')}</div>}
                        </td>
                        <td>
                          <Badge tone={STATUS_TONE[r.status]}>{t(`compare.status.${r.status}`)}</Badge>
                        </td>
                        {r.isPath && r.status === 'changed' && !r.sensitive ? (
                          <td colSpan={2} className="small">
                            <PathDiff before={r.leftExpanded ?? r.left} after={r.rightExpanded ?? r.right} />
                          </td>
                        ) : (
                          <>
                            <td className="small">
                              <Value v={r.left} present={r.leftPresent} sensitive={r.sensitive} />
                            </td>
                            <td className="small">
                              <Value v={r.right} present={r.rightPresent} sensitive={r.sensitive} />
                            </td>
                          </>
                        )}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </Card>
        )}
      </div>
    </div>
  );
}
