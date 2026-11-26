import { useState } from 'react';
import { save as saveDialog } from '@tauri-apps/plugin-dialog';
import { useT, errorText } from '../../i18n/index.jsx';
import { useSettings } from '../../app/settings.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../ui/Toast.jsx';
import { Modal } from '../ui/Modal.jsx';
import { Button, Callout, Checkbox, Field, Segmented, Toggle } from '../ui/primitives.jsx';

const EXT = { json: 'json', csv: 'csv', env: 'env', text: 'txt' };

export function defaultFileName(request, format) {
  const stamp = new Date().toISOString().slice(0, 10);
  const base = request.kind === 'variables' ? 'envora-degiskenler' : request.kind === 'path' ? `envora-path-${request.scope}` : `envora-${request.kind}-${request.id}`;
  return format === 'env' ? `${base}-${stamp}.env` : `${base}-${stamp}.${EXT[format]}`;
}

/**
 * Dışa aktarma penceresi. `preset` verilirse (ör. {kind:'profile', id}) yalnızca o öğe dışa aktarılır.
 * Hassas değerler varsayılan olarak dahil edilmez.
 */
export default function ExportDialog({ preset, onClose }) {
  const t = useT();
  const toast = useToast();
  const { settings } = useSettings();
  const [scopes, setScopes] = useState({ user: true, machine: false, process: false });
  const [format, setFormat] = useState(preset?.kind === 'path' ? 'text' : 'json');
  const [include, setInclude] = useState(!settings.security.excludeSensitiveOnExport);
  const [ack, setAck] = useState(false);
  const [busy, setBusy] = useState(false);
  const kind = preset?.kind || 'variables';

  const request =
    kind === 'variables'
      ? { kind, scopes: Object.keys(scopes).filter((s) => scopes[s]), names: null, format }
      : kind === 'path'
        ? { kind, scope: preset.scope, format }
        : { kind, id: preset.id };
  const fmt = kind === 'variables' || kind === 'path' ? format : 'json';
  const valid = (kind !== 'variables' || request.scopes.length > 0) && (!include || ack);

  const run = async () => {
    const dest = await saveDialog({ defaultPath: defaultFileName(request, fmt), filters: [{ name: fmt.toUpperCase(), extensions: [EXT[fmt]] }] });
    if (!dest) return;
    setBusy(true);
    try {
      const r = await api.exportWrite(request, dest, include);
      toast.ok(t('transfer.exported', { count: r.count }) + (r.excludedSensitive ? ` ${t('transfer.excludedSensitive', { count: r.excludedSensitive })}` : ''));
      if (r.containsSecrets) toast.warn(t('transfer.fileHasSecrets'));
      onClose();
    } catch (e) {
      toast.error(errorText(t, e));
      setBusy(false);
    }
  };

  return (
    <Modal
      title={t('transfer.exportTitle')}
      onClose={busy ? undefined : onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant={include ? 'danger' : 'primary'} icon="download" onClick={run} disabled={busy || !valid}>
            {t('transfer.chooseFile')}
          </Button>
        </>
      }
    >
      <div className="stack">
        {kind === 'variables' && (
          <>
            <Field label={t('snapshots.scopes')}>
              <div className="row">
                {['user', 'machine', 'process'].map((s) => (
                  <Checkbox key={s} checked={scopes[s]} onChange={(v) => setScopes({ ...scopes, [s]: v })}>
                    {t(`scope.${s}`)}
                  </Checkbox>
                ))}
              </div>
            </Field>
            <Field label={t('transfer.format')} hint={t(`transfer.formatHint.${format}`)}>
              <Segmented
                label={t('transfer.format')}
                value={format}
                onChange={setFormat}
                options={[
                  { value: 'json', label: 'JSON' },
                  { value: 'csv', label: 'CSV' },
                  { value: 'env', label: '.env' },
                ]}
              />
            </Field>
          </>
        )}
        {kind === 'path' && (
          <Field label={t('transfer.format')}>
            <Segmented
              label={t('transfer.format')}
              value={format}
              onChange={setFormat}
              options={[
                { value: 'text', label: t('transfer.textList') },
                { value: 'json', label: 'JSON' },
              ]}
            />
          </Field>
        )}
        {kind !== 'variables' && kind !== 'path' && <p className="muted small">{t('transfer.jsonOnly')}</p>}
        {kind !== 'path' && (
          <>
            <div className="row between">
              <div>
                <div className="setting-label">{t('transfer.includeSensitive')}</div>
                <div className="field-hint">{t('transfer.includeSensitiveHint')}</div>
              </div>
              <Toggle
                label={t('transfer.includeSensitive')}
                checked={include}
                onChange={(v) => {
                  setInclude(v);
                  setAck(false);
                }}
              />
            </div>
            {include && (
              <Callout tone="danger" title={t('transfer.plaintextTitle')}>
                {t('transfer.plaintextBody')}
                <div style={{ marginTop: 8 }}>
                  <Checkbox checked={ack} onChange={setAck}>
                    {t('transfer.plaintextAck')}
                  </Checkbox>
                </div>
              </Callout>
            )}
          </>
        )}
      </div>
    </Modal>
  );
}
