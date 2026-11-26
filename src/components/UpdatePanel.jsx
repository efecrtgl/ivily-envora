import { useState } from 'react';
import { Channel } from '@tauri-apps/api/core';
import { useT, errorText } from '../i18n/index.jsx';
import { api, inTauri } from '../lib/api.js';
import { useToast } from './ui/Toast.jsx';
import { ConfirmDialog } from './ui/Modal.jsx';
import { Badge, Button, Callout, Meter } from './ui/primitives.jsx';

export const LAST_CHECK_KEY = 'envora.lastUpdateCheck';
const DAY = 24 * 60 * 60 * 1000;

/** Açılış denetimi gerekiyor mu? (ayar açıksa ve son denetimden bu yana 24 saat geçtiyse) */
export function shouldCheckOnStartup(enabled, lastCheck, now = Date.now()) {
  if (!enabled) return false;
  const last = Number(lastCheck) || 0;
  return now - last >= DAY;
}

export function markChecked() {
  try {
    localStorage.setItem(LAST_CHECK_KEY, String(Date.now()));
  } catch {
    /* yerel depolama yoksa yalnızca bu oturum */
  }
}

/** Güncelleme denetimi ve kullanıcı onaylı kurulum. */
export default function UpdatePanel({ initial }) {
  const t = useT();
  const toast = useToast();
  const [state, setState] = useState({ checking: false, info: initial ?? undefined, error: null });
  const [confirm, setConfirm] = useState(false);
  const [progress, setProgress] = useState(null);

  const check = async () => {
    setState({ checking: true, info: undefined, error: null });
    try {
      const info = await api.updateCheck();
      markChecked();
      setState({ checking: false, info, error: null });
    } catch (error) {
      setState({ checking: false, info: undefined, error });
    }
  };

  const install = async () => {
    setConfirm(false);
    setProgress({ received: 0, total: null, installing: false });
    const channel = new Channel();
    channel.onmessage = (m) => {
      if (m.event === 'downloading') setProgress({ received: m.received, total: m.total, installing: false });
      if (m.event === 'installing') setProgress((p) => ({ ...p, installing: true }));
    };
    try {
      await api.updateInstall(channel);
    } catch (e) {
      setProgress(null);
      toast.error(errorText(t, e));
    }
  };

  const info = state.info;
  return (
    <div className="stack tight">
      <div className="row">
        <Button icon="refresh" onClick={check} disabled={!inTauri() || state.checking || !!progress}>
          {state.checking ? t('updates.checking') : t('updates.check')}
        </Button>
        {info === null && <Badge tone="ok">{t('updates.upToDate')}</Badge>}
        {info && <Badge tone="info">{t('updates.available', { version: info.version })}</Badge>}
      </div>
      {state.error && <Callout tone="danger">{t('updates.failed')} {errorText(t, state.error)}</Callout>}
      {info && (
        <div className="card" style={{ padding: 14 }}>
          <div className="row between">
            <strong>
              {info.currentVersion} → {info.version}
            </strong>
            <span className="xsmall faint">{info.date || ''}</span>
          </div>
          <div className="field-label" style={{ marginTop: 10 }}>
            {t('updates.notes')}
          </div>
          <pre className="small selectable" style={{ whiteSpace: 'pre-wrap', margin: '6px 0 10px', fontFamily: 'var(--font-sans)', maxHeight: 220, overflow: 'auto' }}>
            {info.notes || t('updates.noNotes')}
          </pre>
          {progress ? (
            <div className="stack tight">
              <Meter value={progress.received} max={progress.total || progress.received || 1} />
              <span className="xsmall muted">{progress.installing ? t('updates.installing') : t('updates.downloading', { mb: (progress.received / 1048576).toFixed(1) })}</span>
            </div>
          ) : (
            <Button variant="primary" icon="download" onClick={() => setConfirm(true)}>
              {t('updates.install')}
            </Button>
          )}
        </div>
      )}
      <p className="field-hint">{t('updates.securityNote')}</p>
      {confirm && (
        <ConfirmDialog title={t('updates.confirmTitle', { version: info.version })} confirmLabel={t('updates.install')} onCancel={() => setConfirm(false)} onConfirm={install}>
          <p className="muted">{t('updates.confirmBody')}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}
