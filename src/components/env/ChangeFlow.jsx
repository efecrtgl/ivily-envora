import { useEffect, useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { useSettings } from '../../app/settings.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../ui/Toast.jsx';
import { Modal } from '../ui/Modal.jsx';
import { Badge, Button, Callout, Loading } from '../ui/primitives.jsx';
import ScopeBadge from './ScopeBadge.jsx';

const ACTION_TONE = { create: 'ok', update: 'warn', delete: 'danger', unchanged: undefined };

/** İki PATH değeri arasındaki girdi farkı (sıra değişikliği ayrıca belirtilir). */
export function pathEntryDiff(before, after) {
  const split = (v) => (v ? v.split(';') : []);
  const key = (e) => e.trim().replace(/\//g, '\\').replace(/\\+$/, '').toLowerCase();
  const b = split(before);
  const a = split(after);
  const bk = new Set(b.map(key));
  const ak = new Set(a.map(key));
  const added = a.filter((e) => !bk.has(key(e)));
  const removed = b.filter((e) => !ak.has(key(e)));
  const common = (xs, set) => xs.map(key).filter((k) => set.has(k));
  const reordered = common(b, ak).join('|') !== common(a, bk).join('|');
  return { added, removed, reordered };
}

export function PathDiff({ before, after }) {
  const t = useT();
  const d = pathEntryDiff(before, after);
  return (
    <div className="mono xsmall stack tight" style={{ gap: 2 }}>
      {d.added.map((e, i) => (
        <div key={`a${i}`} className="diff-new break">
          + {e || t('pathmgr.emptyEntry')}
        </div>
      ))}
      {d.removed.map((e, i) => (
        <div key={`r${i}`} className="diff-old break">
          − {e || t('pathmgr.emptyEntry')}
        </div>
      ))}
      {d.reordered && (
        <div className="faint" style={{ fontFamily: 'var(--font-sans)' }}>
          {t('change.pathReordered')}
        </div>
      )}
      {!d.added.length && !d.removed.length && !d.reordered && <span className="faint">—</span>}
    </div>
  );
}

function ValueCell({ value, sensitive, kind }) {
  const t = useT();
  if (sensitive) return <span className="mask faint">••••••••</span>;
  if (value == null) return <span className="faint">—</span>;
  if (value === '') return <span className="faint">{t('vars.emptyValue')}</span>;
  return (
    <span className="mono break selectable" title={kind ? t(`kind.${kind}`) : undefined}>
      {value}
    </span>
  );
}

/** Önizleme tablosu (onay penceresinden bağımsız olarak da kullanılabilir). */
export function PreviewTable({ preview }) {
  const t = useT();
  return (
    <div className="table-wrap" style={{ maxHeight: 340, border: '1px solid var(--border)', borderRadius: 10 }}>
      <table className="table fixed top">
        <colgroup>
          <col style={{ width: '26%' }} />
          <col style={{ width: 110 }} />
          <col />
          <col />
        </colgroup>
        <thead>
          <tr>
            <th>{t('common.name')}</th>
            <th>{t('change.action')}</th>
            <th>{t('change.before')}</th>
            <th>{t('change.after')}</th>
          </tr>
        </thead>
        <tbody>
          {preview.items.map((i) => (
            <tr key={i.name} className={`diff-row ${i.action === 'create' ? 'added' : i.action === 'delete' ? 'removed' : i.action === 'update' ? 'changed' : ''}`}>
              <td className="name" title={i.name}>
                {i.name}
                {i.warnings.length > 0 && (
                  <div className="xsmall" style={{ color: 'var(--warn)', fontFamily: 'var(--font-sans)', fontWeight: 400, whiteSpace: 'normal' }}>
                    {i.warnings.map((w) => t(`change.warn.${w}`)).join(' ')}
                  </div>
                )}
              </td>
              <td>
                <Badge tone={ACTION_TONE[i.action]}>{t(`change.actions.${i.action}`)}</Badge>
              </td>
              {i.name.toUpperCase() === 'PATH' && i.action === 'update' && !i.sensitive ? (
                <td className="small" colSpan={2}>
                  <PathDiff before={i.before} after={i.after} />
                </td>
              ) : (
                <>
                  <td className="small">
                    <ValueCell value={i.before} sensitive={i.sensitive && i.kindBefore != null} kind={i.kindBefore} />
                  </td>
                  <td className="small">
                    <ValueCell value={i.after} sensitive={i.sensitive && i.kindAfter != null} kind={i.kindAfter} />
                    {i.kindBefore && i.kindAfter && i.kindBefore !== i.kindAfter && (
                      <div className="xsmall faint">
                        {t(`kind.${i.kindBefore}`)} → {t(`kind.${i.kindAfter}`)}
                      </div>
                    )}
                  </td>
                </>
              )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/**
 * Değişiklik akışı: plan → önizleme → onay → uygulama.
 * `plan`: { scope, ops }. `onDone(outcome)` başarılı uygulamadan sonra çağrılır.
 */
export default function ChangeFlow({ plan, context, title, onClose, onDone }) {
  const t = useT();
  const toast = useToast();
  const { settings } = useSettings();
  const [preview, setPreview] = useState(null);
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    api
      .envPlanPreview(plan)
      .then((p) => alive && setPreview(p))
      .catch((e) => alive && setError(e));
    return () => {
      alive = false;
    };
  }, [plan]);

  const apply = async () => {
    setBusy(true);
    setError(null);
    try {
      const out = await api.envPlanApply(plan, context);
      if (settings.notifications.showSuccess) toast.ok(t('change.applied', { count: out.applied }));
      if (settings.notifications.restartHint && out.applied > 0) toast.info(t('change.restartHint'));
      onDone?.(out);
      onClose();
    } catch (e) {
      if (e?.code === 'cancelled') toast.warn(t('change.elevationCancelled'));
      else setError(e);
      setBusy(false);
    }
  };

  const hasCritical = preview?.items.some((i) => i.warnings.includes('critical') || i.warnings.includes('pathDelete'));

  return (
    <Modal
      wide
      title={title || t('change.title')}
      onClose={busy ? undefined : onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant={hasCritical ? 'danger' : 'primary'} onClick={apply} disabled={busy || !preview || preview.changes === 0}>
            {busy && <span className="spinner" style={{ width: 14, height: 14 }} />}
            {preview?.requiresElevation ? t('change.applyElevated') : t('change.apply')}
          </Button>
        </>
      }
    >
      {!preview && !error && <Loading label={t('change.previewing')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      {preview && (
        <div className="stack">
          <div className="row">
            <span className="muted small">{t('change.affectedScope')}:</span>
            <ScopeBadge scope={preview.scope} />
            <span className="spacer" />
            <span className="small muted">{t('change.count', { count: preview.changes })}</span>
          </div>
          <PreviewTable preview={preview} />
          {preview.changes === 0 && <Callout>{t('change.nothing')}</Callout>}
          {preview.requiresElevation && (
            <Callout tone="warn" icon="shield">
              {t('change.elevationNote')}
            </Callout>
          )}
          {hasCritical && <Callout tone="danger">{t('change.criticalNote')}</Callout>}
          <Callout tone="info">
            {settings.backup.autoSnapshotBeforeChanges ? t('change.backupNote') : t('change.noBackupNote')} {t('scope.restartNote')}
          </Callout>
        </div>
      )}
    </Modal>
  );
}
