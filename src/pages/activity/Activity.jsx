import { useCallback, useEffect, useState } from 'react';
import { useNav } from '../../app/navigation.js';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api, inTauri } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog } from '../../components/ui/Modal.jsx';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Empty, Loading, PageHeader, SearchInput, Segmented } from '../../components/ui/primitives.jsx';

const PAGE = 100;
export const CATEGORIES = ['variable', 'path', 'profile', 'project', 'snapshot', 'import', 'export', 'compare', 'analysis', 'discovery', 'template', 'security', 'backup', 'settings', 'ecosystem'];
const STATUS_TONE = { ok: 'ok', failed: 'danger', warning: 'warn' };

/** Çeviri yoksa ham değeri döndüren yardımcı. */
export function label(t, key, fallback) {
  const s = t(key);
  return s === key ? fallback : s;
}

/** Ayrıntı nesnesini okunabilir satırlara çevirir (değer içermez; yalnızca sayılar/kodlar). */
export function detailLines(details) {
  if (!details || typeof details !== 'object') return [];
  return Object.entries(details)
    .filter(([, v]) => v !== null && v !== undefined && v !== '')
    .map(([k, v]) => [k, typeof v === 'object' ? JSON.stringify(v) : String(v)]);
}

export function EntryLine({ e, compact }) {
  const t = useT();
  const lang = useLang();
  const [open, setOpen] = useState(false);
  const lines = detailLines(e.details);
  return (
    <div className="list-item" style={{ alignItems: 'flex-start' }}>
      <span className={`dot ${e.status === 'ok' ? 'ok' : e.status === 'failed' ? 'danger' : 'warn'}`} style={{ marginTop: 6 }} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div className="row">
          <Badge>{label(t, `activity.category.${e.category}`, e.category)}</Badge>
          <span className="small">{label(t, `activity.actions.${e.action}`, e.action)}</span>
          {e.target && <span className="mono small truncate" style={{ maxWidth: 320 }} title={e.target}>{e.target}</span>}
          {e.status !== 'ok' && <Badge tone={STATUS_TONE[e.status]}>{t(`activity.status.${e.status}`)}</Badge>}
        </div>
        {open && lines.length > 0 && (
          <dl className="kv xsmall" style={{ marginTop: 6 }}>
            {lines.map(([k, v]) => (
              <div key={k} style={{ display: 'contents' }}>
                <dt>{k}</dt>
                <dd className="mono break">{v}</dd>
              </div>
            ))}
          </dl>
        )}
      </div>
      <span className="xsmall faint" style={{ whiteSpace: 'nowrap' }}>
        {formatDate(lang, e.ts)}
      </span>
      {!compact && lines.length > 0 && (
        <Button size="sm" variant="ghost" iconOnly icon={open ? 'chevronDown' : 'chevronRight'} title={t('common.details')} onClick={() => setOpen(!open)} />
      )}
    </div>
  );
}

export default function Activity() {
  const t = useT();
  const toast = useToast();
  const { navigate } = useNav();
  const [filter, setFilter] = useState({ search: '', category: '', status: '', order: 'desc' });
  const [items, setItems] = useState([]);
  const [state, setState] = useState({ loading: true, error: null, more: false });
  const [confirmClear, setConfirmClear] = useState(false);

  const load = useCallback(
    async (append) => {
      if (!inTauri()) return setState({ loading: false, error: null, more: false });
      setState((s) => ({ ...s, loading: true, error: null }));
      try {
        const last = append ? items[items.length - 1] : null;
        const rows = await api.activityList({ ...filter, limit: PAGE, beforeId: last ? last.id : null });
        setItems(append ? [...items, ...rows] : rows);
        setState({ loading: false, error: null, more: rows.length === PAGE });
      } catch (error) {
        setState({ loading: false, error, more: false });
      }
    },
    [filter, items],
  );

  useEffect(() => {
    const id = setTimeout(() => load(false), 200);
    return () => clearTimeout(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filter]);

  const set = (patch) => setFilter((f) => ({ ...f, ...patch }));

  return (
    <div className="page">
      <PageHeader
        title={t('activity.title')}
        subtitle={t('activity.subtitle')}
        actions={
          <>
            <Button variant="ghost" icon="settings" onClick={() => navigate('settings')}>
              {t('activity.retention')}
            </Button>
            <Button variant="danger" icon="trash" onClick={() => setConfirmClear(true)} disabled={!items.length}>
              {t('activity.clear')}
            </Button>
          </>
        }
      />
      <Card flush>
        <div className="toolbar">
          <SearchInput value={filter.search} onChange={(search) => set({ search })} placeholder={t('common.search')} />
          <select className="select" style={{ width: 170 }} value={filter.category} onChange={(e) => set({ category: e.target.value })} aria-label={t('vars.category')}>
            <option value="">{t('vars.allCategories')}</option>
            {CATEGORIES.map((c) => (
              <option key={c} value={c}>
                {t(`activity.category.${c}`)}
              </option>
            ))}
          </select>
          <Segmented
            label={t('importer.status')}
            value={filter.status}
            onChange={(status) => set({ status })}
            options={['', 'ok', 'warning', 'failed'].map((s) => ({ value: s, label: s ? t(`activity.status.${s}`) : t('common.all') }))}
          />
          <Button size="sm" variant="ghost" icon={filter.order === 'desc' ? 'arrowDown' : 'arrowUp'} onClick={() => set({ order: filter.order === 'desc' ? 'asc' : 'desc' })}>
            {filter.order === 'desc' ? t('activity.newest') : t('activity.oldest')}
          </Button>
        </div>
        {state.error && (
          <div style={{ padding: 20 }}>
            <Callout tone="danger">{errorText(t, state.error)}</Callout>
          </div>
        )}
        {state.loading && !items.length && <Loading label={t('common.loading')} />}
        {!state.loading && !items.length && !state.error && <Empty icon="activity" title={t('activity.empty')}>{t('activity.emptyHint')}</Empty>}
        <div className="list">
          {items.map((e) => (
            <EntryLine key={e.id} e={e} />
          ))}
        </div>
        {state.more && (
          <div style={{ padding: 16, textAlign: 'center' }}>
            <Button size="sm" onClick={() => load(true)} disabled={state.loading}>
              {t('activity.loadMore')}
            </Button>
          </div>
        )}
      </Card>
      <p className="xsmall faint" style={{ marginTop: 12 }}>
        <Icon name="lock" size={12} /> {t('activity.privacy')}
      </p>
      {confirmClear && (
        <ConfirmDialog
          danger
          title={t('activity.clearTitle')}
          confirmLabel={t('activity.clear')}
          onCancel={() => setConfirmClear(false)}
          onConfirm={async () => {
            try {
              const n = await api.activityClear();
              toast.ok(t('activity.cleared', { count: n }));
              setConfirmClear(false);
              load(false);
            } catch (e) {
              toast.error(errorText(t, e));
            }
          }}
        >
          <p className="muted">{t('activity.clearBody')}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}
