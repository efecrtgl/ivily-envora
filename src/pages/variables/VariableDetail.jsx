import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useAsync } from '../../lib/useAsync.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { Modal } from '../../components/ui/Modal.jsx';
import { Badge, Button, Callout, Loading, Segmented } from '../../components/ui/primitives.jsx';
import ScopeBadge from '../../components/env/ScopeBadge.jsx';
import SecretValue from '../../components/env/SecretValue.jsx';
import { useCopyValue } from '../../components/env/useCopyValue.jsx';

function SensitivityControl({ name, entry, onChanged }) {
  const t = useT();
  const toast = useToast();
  const v = entry.view;
  const value = v.overridden ? (v.sensitive ? 'yes' : 'no') : 'auto';
  const change = async (next) => {
    try {
      await api.sensitivitySet(name, next === 'auto' ? null : next === 'yes');
      onChanged();
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };
  return (
    <Segmented
      label={t('vars.sensitivity')}
      value={value}
      onChange={change}
      options={[
        { value: 'auto', label: t('vars.sensAuto') },
        { value: 'yes', label: t('vars.sensYes') },
        { value: 'no', label: t('vars.sensNo') },
      ]}
    />
  );
}

export default function VariableDetail({ name, onClose, onChanged, actions }) {
  const t = useT();
  const { loading, error, data, reload } = useAsync(() => api.envDetail(name), [name]);
  const { copy, dialog } = useCopyValue();

  const refresh = () => {
    reload();
    onChanged?.();
  };

  return (
    <Modal title={name} onClose={onClose} wide footer={<Button onClick={onClose}>{t('common.close')}</Button>}>
      {loading && !data && <Loading label={t('common.loading')} />}
      {error && <Callout tone="danger">{errorText(t, error)}</Callout>}
      {data && (
        <div className="stack">
          {data.processDiffers && <Callout tone="info">{t('vars.processDiffers')}</Callout>}
          {data.effectiveScope && (
            <p className="small muted">
              {t('vars.effectiveScope')}: <strong>{t(`scope.${data.effectiveScope}`)}</strong>
              {data.name.toUpperCase() === 'PATH' && ` — ${t('vars.pathMerged')}`}
            </p>
          )}
          {data.entries.map((e) => {
            const v = e.view;
            return (
              <div key={v.scope} className="card" style={{ padding: 16 }}>
                <div className="row between" style={{ marginBottom: 10 }}>
                  <div className="row">
                    <ScopeBadge scope={v.scope} />
                    <Badge>{t(`kind.${v.kind}`)}</Badge>
                    {v.sensitive && (
                      <Badge tone="warn" icon="lock" title={v.sensitiveReason ? t(`reason.${v.sensitiveReason}`) : ''}>
                        {t('vars.sensitive')}
                      </Badge>
                    )}
                    {v.readOnly && <Badge>{t('vars.readOnly')}</Badge>}
                  </div>
                  <div className="row">
                    {actions?.(v, refresh)}
                    <Button size="sm" variant="ghost" icon="copy" onClick={() => copy(v)} disabled={v.kind === 'other'}>
                      {t('common.copy')}
                    </Button>
                  </div>
                </div>
                <div className="field-label">{t('common.value')}</div>
                <div style={{ margin: '4px 0 10px' }}>
                  <SecretValue v={v} multiline />
                </div>
                {e.expanded != null && e.expanded !== v.value && (
                  <>
                    <div className="field-label">{t('vars.expanded')}</div>
                    <div className="mono small break selectable muted" style={{ margin: '4px 0 10px' }}>
                      {e.expanded}
                    </div>
                  </>
                )}
                {v.issues.length > 0 && (
                  <ul className="small" style={{ margin: '0 0 10px', paddingLeft: 18 }}>
                    {v.issues.map((i) => (
                      <li key={i} style={{ color: 'var(--warn)' }}>
                        {t(`issue.${i}`)}
                      </li>
                    ))}
                  </ul>
                )}
                {v.sensitive && v.sensitiveReason && !v.overridden && (
                  <p className="field-hint" style={{ marginBottom: 10 }}>
                    {t('vars.detectedBecause')}: {t(`reason.${v.sensitiveReason}`)}
                  </p>
                )}
                <div className="row between">
                  <span className="small muted">{t('vars.sensitivity')}</span>
                  <SensitivityControl name={data.name} entry={e} onChanged={refresh} />
                </div>
              </div>
            );
          })}
        </div>
      )}
      {dialog}
    </Modal>
  );
}
