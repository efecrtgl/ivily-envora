import { useMemo, useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { filterVars, countBy } from '../../lib/envFilter.js';
import { useToast } from '../../components/ui/Toast.jsx';
import Icon from '../../components/ui/Icon.jsx';
import {
  Badge,
  Button,
  Callout,
  Card,
  Checkbox,
  Empty,
  Loading,
  PageHeader,
  SearchInput,
  Tabs,
} from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import ScopeHelp from '../../components/env/ScopeHelp.jsx';
import SecretValue from '../../components/env/SecretValue.jsx';
import { useCopyValue, copyPlainText } from '../../components/env/useCopyValue.jsx';
import VariableDetail from './VariableDetail.jsx';
import VariableEditor from '../../components/env/VariableEditor.jsx';
import ChangeFlow from '../../components/env/ChangeFlow.jsx';

const CATEGORIES = ['all', 'path', 'development', 'credentials', 'user', 'system', 'other'];
const MAX_ROWS = 500;

export default function Variables() {
  const t = useT();
  const toast = useToast();
  const { loading, error, data, reload } = useAsync(() => api.envList(), []);
  const [filter, setFilter] = useState({ search: '', scope: 'all', category: 'all', onlySensitive: false, onlyIssues: false, sort: 'name' });
  const [selected, setSelected] = useState(null);
  const { copy, dialog } = useCopyValue();
  const [editor, setEditor] = useState(null); // { mode, initial }
  const [flow, setFlow] = useState(null); // { plan, title }
  const overview = useAsync(() => api.envOverview(), []);
  const canWriteMachine = !!overview.data?.canWriteMachine;

  const startPlan = (plan, title) => {
    setEditor(null);
    setFlow({ plan, title });
  };

  const entryActions = (v) => {
    if (v.scope === 'process') {
      return (
        <Button size="sm" variant="ghost" icon="copy" onClick={() => setEditor({ mode: 'duplicate', initial: v })}>
          {t('editor.persist')}
        </Button>
      );
    }
    if (v.readOnly) return null;
    return (
      <>
        <Button size="sm" variant="ghost" iconOnly icon="edit" title={t('common.edit')} onClick={() => setEditor({ mode: 'edit', initial: v })} />
        <Button size="sm" variant="ghost" iconOnly icon="copy" title={t('common.duplicate')} onClick={() => setEditor({ mode: 'duplicate', initial: v })} />
        <Button size="sm" variant="ghost" iconOnly icon="tag" title={t('common.rename')} onClick={() => setEditor({ mode: 'rename', initial: v })} />
        <Button
          size="sm"
          variant="ghost"
          iconOnly
          icon="trash"
          title={t('common.delete')}
          onClick={() => startPlan({ scope: v.scope, ops: [{ op: 'delete', name: v.name }] }, t('editor.deleteTitle', { name: v.name }))}
        />
      </>
    );
  };

  const vars = data || [];
  const rows = useMemo(() => filterVars(vars, filter), [vars, filter]);
  const counts = useMemo(() => countBy(vars, 'scope'), [vars]);
  const set = (patch) => setFilter((f) => ({ ...f, ...patch }));

  const copyName = async (name) => {
    try {
      await copyPlainText(name);
      toast.ok(t('common.copied'));
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  return (
    <div className="page">
      <PageHeader
        title={t('vars.title')}
        subtitle={t('vars.subtitle')}
        actions={
          <>
            <Button icon="refresh" onClick={reload} disabled={loading}>
              {t('common.refresh')}
            </Button>
            <Button variant="primary" icon="plus" onClick={() => setEditor({ mode: 'create', initial: {} })}>
              {t('editor.new')}
            </Button>
          </>
        }
      />
      <div className="stack">
        <ScopeHelp />
        <Card flush>
          <div className="toolbar">
            <SearchInput value={filter.search} onChange={(search) => set({ search })} placeholder={t('vars.searchPlaceholder')} />
            <select className="select" style={{ width: 170 }} value={filter.category} onChange={(e) => set({ category: e.target.value })} aria-label={t('vars.category')}>
              {CATEGORIES.map((c) => (
                <option key={c} value={c}>
                  {c === 'all' ? t('vars.allCategories') : t(`category.${c}`)}
                </option>
              ))}
            </select>
            <select className="select" style={{ width: 150 }} value={filter.sort} onChange={(e) => set({ sort: e.target.value })} aria-label={t('vars.sort')}>
              <option value="name">{t('vars.sortName')}</option>
              <option value="nameDesc">{t('vars.sortNameDesc')}</option>
              <option value="scope">{t('vars.sortScope')}</option>
            </select>
            <Checkbox checked={filter.onlySensitive} onChange={(onlySensitive) => set({ onlySensitive })}>
              {t('vars.onlySensitive')}
            </Checkbox>
            <Checkbox checked={filter.onlyIssues} onChange={(onlyIssues) => set({ onlyIssues })}>
              {t('vars.onlyIssues')}
            </Checkbox>
          </div>
          <div style={{ padding: '0 20px' }}>
            <Tabs
              value={filter.scope}
              onChange={(scope) => set({ scope })}
              tabs={[
                { value: 'all', label: t('common.all'), count: vars.length },
                { value: 'user', label: t('scope.user'), count: counts.user || 0 },
                { value: 'machine', label: t('scope.machine'), count: counts.machine || 0 },
                { value: 'process', label: t('scope.process'), count: counts.process || 0 },
              ]}
            />
          </div>

          {loading && !data && <Loading label={t('common.loading')} />}
          {error && (
            <div style={{ padding: 20 }}>
              <Callout tone="danger">{errorText(t, error)}</Callout>
            </div>
          )}
          {data && rows.length === 0 && (
            <Empty icon="search" title={t('vars.noResults')}>
              {t('vars.noResultsHint')}
            </Empty>
          )}
          {rows.length > 0 && (
            <div className="table-wrap" style={{ maxHeight: 'calc(100vh - 330px)' }}>
              <table className="table fixed">
                <colgroup>
                  <col style={{ width: '30%' }} />
                  <col style={{ width: 118 }} />
                  <col />
                  <col style={{ width: 170 }} />
                  <col style={{ width: 84 }} />
                </colgroup>
                <thead>
                  <tr>
                    <th>{t('common.name')}</th>
                    <th>{t('common.scope')}</th>
                    <th>{t('common.value')}</th>
                    <th>{t('vars.category')}</th>
                    <th className="actions">
                      <span className="sr-only">{t('common.actions')}</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {rows.slice(0, MAX_ROWS).map((v) => (
                    <tr key={`${v.scope}:${v.name}`} className="clickable" onClick={() => setSelected(v.name)}>
                      <td className="name" title={v.name}>
                        <span className="row" style={{ flexWrap: 'nowrap' }}>
                          <span className="truncate">{v.name}</span>
                          {v.sensitive && <Icon name="lock" size={12} title={t('vars.sensitive')} />}
                          {v.issues.length > 0 && (
                            <span style={{ color: 'var(--warn)' }}>
                              <Icon name="alert" size={12} title={v.issues.map((i) => t(`issue.${i}`)).join('\n')} />
                            </span>
                          )}
                        </span>
                      </td>
                      <td>
                        <ScopeBadge scope={v.scope} />
                      </td>
                      <td className="value">
                        <SecretValue v={v} />
                      </td>
                      <td>
                        <Badge>{t(`category.${v.category}`)}</Badge>
                        {v.kind === 'expand' && (
                          <>
                            {' '}
                            <Badge title={t('kind.expandHelp')}>%</Badge>
                          </>
                        )}
                      </td>
                      <td className="actions" onClick={(e) => e.stopPropagation()}>
                        <Button size="sm" variant="ghost" iconOnly icon="copy" title={t('vars.copyValue')} onClick={() => copy(v)} disabled={v.kind === 'other'} />
                        <Button size="sm" variant="ghost" iconOnly icon="tag" title={t('vars.copyName')} onClick={() => copyName(v.name)} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {rows.length > MAX_ROWS && <p className="small muted" style={{ padding: 12 }}>{t('vars.truncated', { shown: MAX_ROWS, total: rows.length })}</p>}
            </div>
          )}
        </Card>
      </div>
      {selected && !editor && !flow && (
        <VariableDetail name={selected} onClose={() => setSelected(null)} onChanged={reload} actions={entryActions} />
      )}
      {editor && (
        <VariableEditor
          mode={editor.mode}
          initial={editor.initial}
          canWriteMachine={canWriteMachine}
          onClose={() => setEditor(null)}
          onPlan={(plan) => startPlan(plan)}
        />
      )}
      {flow && (
        <ChangeFlow
          plan={flow.plan}
          title={flow.title}
          onClose={() => setFlow(null)}
          onDone={() => {
            setSelected(null);
            reload();
            overview.reload();
          }}
        />
      )}
      {dialog}
    </div>
  );
}
