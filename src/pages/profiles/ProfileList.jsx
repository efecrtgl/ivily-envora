import { useMemo, useState } from 'react';
import { useT, errorText, useLang, formatDate } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import Icon from '../../components/ui/Icon.jsx';
import { Badge, Button, Callout, Card, Checkbox, Empty, Loading, PageHeader, SearchInput } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import ProfileEditor from './ProfileEditor.jsx';

export function filterProfiles(list, { search, category }) {
  const q = (search || '').trim().toLowerCase();
  return list.filter((p) => {
    if (category && category !== 'all' && p.category !== category) return false;
    if (!q) return true;
    return (
      p.name.toLowerCase().includes(q) ||
      p.description.toLowerCase().includes(q) ||
      p.tags.some((tag) => tag.toLowerCase().includes(q))
    );
  });
}

export default function ProfileList({ onOpen }) {
  const t = useT();
  const lang = useLang();
  const [archived, setArchived] = useState(false);
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('all');
  const [creating, setCreating] = useState(false);
  const { loading, error, data, reload } = useAsync(() => api.profilesList(archived), [archived]);

  const list = data || [];
  const categories = useMemo(() => [...new Set(list.map((p) => p.category).filter(Boolean))].sort(), [list]);
  const rows = useMemo(() => filterProfiles(list, { search, category }), [list, search, category]);

  return (
    <div className="page">
      <PageHeader
        title={t('profiles.title')}
        subtitle={t('profiles.subtitle')}
        actions={
          <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
            {t('profiles.new')}
          </Button>
        }
      />
      <div className="toolbar">
        <SearchInput value={search} onChange={setSearch} placeholder={t('profiles.search')} />
        <select className="select" style={{ width: 180 }} value={category} onChange={(e) => setCategory(e.target.value)} aria-label={t('profiles.category')}>
          <option value="all">{t('vars.allCategories')}</option>
          {categories.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
        <Checkbox checked={archived} onChange={setArchived}>
          {t('profiles.showArchived')}
        </Checkbox>
      </div>
      {loading && !data && <Loading label={t('common.loading')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      {data && list.length === 0 && (
        <Card>
          <Empty
            icon="profile"
            title={t('profiles.emptyTitle')}
            action={
              <Button variant="primary" icon="plus" onClick={() => setCreating(true)}>
                {t('profiles.new')}
              </Button>
            }
          >
            {t('profiles.emptyBody')}
          </Empty>
        </Card>
      )}
      {list.length > 0 && rows.length === 0 && <Empty icon="search" title={t('vars.noResults')} />}
      <div className="grid cols-3">
        {rows.map((p) => (
          <button key={p.id} type="button" className="card profile-card" onClick={() => onOpen(p.id)}>
            <div className="row between" style={{ marginBottom: 6 }}>
              <h3 className="truncate" style={{ fontSize: 'var(--fs-lg)' }}>
                {p.name}
              </h3>
              {p.status === 'archived' ? <Badge icon="archive">{t('profiles.archived')}</Badge> : p.hasSecrets && <Icon name="lock" size={14} title={t('vars.sensitive')} />}
            </div>
            <p className="small muted clamp-2">{p.description || t('profiles.noDescription')}</p>
            <div className="row" style={{ marginTop: 10 }}>
              {p.category && <Badge>{p.category}</Badge>}
              {p.tags.slice(0, 3).map((tag) => (
                <Badge key={tag} icon="tag">
                  {tag}
                </Badge>
              ))}
            </div>
            <div className="row between xsmall faint" style={{ marginTop: 12 }}>
              <span>
                {t('profiles.counts', { vars: p.varCount, paths: p.pathCount })} · v{p.version}
              </span>
              <span>{formatDate(lang, p.updatedAt)}</span>
            </div>
            {p.activeScopes.length > 0 && (
              <div className="row" style={{ marginTop: 10 }}>
                <span className="dot ok" />
                <span className="xsmall">{t('profiles.activeIn')}</span>
                {p.activeScopes.map((s) => (
                  <ScopeBadge key={s} scope={s} />
                ))}
              </div>
            )}
          </button>
        ))}
      </div>
      {creating && (
        <ProfileEditor
          onClose={() => setCreating(false)}
          onSaved={(id) => {
            setCreating(false);
            reload();
            onOpen(id);
          }}
        />
      )}
    </div>
  );
}
