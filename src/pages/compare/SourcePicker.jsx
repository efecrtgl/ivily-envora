import { useT } from '../../i18n/index.jsx';

/** Kaynak nesnesini (ör. {kind:'scope', scope:'user'}) seçim değerine ve geri dönüştürür. */
export function encodeSource(s) {
  if (!s) return '';
  if (s.kind === 'scope') return `scope:${s.scope}`;
  if (s.kind === 'effective') return 'effective';
  if (s.kind === 'import') return `import:${s.token}`;
  return `${s.kind}:${s.id}`;
}

export function decodeSource(v) {
  const [kind, arg] = v.split(':');
  if (kind === 'scope') return { kind, scope: arg };
  if (kind === 'effective') return { kind };
  if (kind === 'import') return { kind, token: v.slice('import:'.length) };
  return { kind, id: Number(arg) };
}

export default function SourcePicker({ value, onChange, profiles, projects, snapshots, label }) {
  const t = useT();
  return (
    <select className="select" value={encodeSource(value)} onChange={(e) => onChange(decodeSource(e.target.value))} aria-label={label}>
      {value?.kind === 'import' && (
        <optgroup label={t('compare.importGroup')}>
          <option value={encodeSource(value)}>{t('compare.importedFile')}</option>
        </optgroup>
      )}
      <optgroup label={t('compare.groupCurrent')}>
        <option value="scope:user">{t('scope.user')}</option>
        <option value="scope:machine">{t('scope.machine')}</option>
        <option value="scope:process">{t('compare.processSource')}</option>
        <option value="effective">{t('compare.effective')}</option>
      </optgroup>
      {profiles?.length > 0 && (
        <optgroup label={t('nav.profiles')}>
          {profiles.map((p) => (
            <option key={p.id} value={`profile:${p.id}`}>
              {p.name}
            </option>
          ))}
        </optgroup>
      )}
      {projects?.length > 0 && (
        <optgroup label={t('nav.projects')}>
          {projects.map((p) => (
            <option key={p.id} value={`project:${p.id}`}>
              {p.name}
            </option>
          ))}
        </optgroup>
      )}
      {snapshots?.length > 0 && (
        <optgroup label={t('compare.snapshots')}>
          {snapshots.map((s) => (
            <option key={s.id} value={`snapshot:${s.id}`}>
              {s.name}
            </option>
          ))}
        </optgroup>
      )}
    </select>
  );
}
