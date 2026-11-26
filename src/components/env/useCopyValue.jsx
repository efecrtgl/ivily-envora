import { useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { useSettings } from '../../app/settings.jsx';
import { useToast } from '../ui/Toast.jsx';
import { ConfirmDialog } from '../ui/Modal.jsx';
import { api } from '../../lib/api.js';

/**
 * Değer kopyalama. Kopyalama arka uçta yapılır (değer arayüze gelmez). Hassas değerler için
 * ayar açıksa önce uyarı gösterilir. `{ copy, dialog }` döndürür; `dialog` bileşen ağacına eklenmelidir.
 */
export function useCopyValue() {
  const t = useT();
  const toast = useToast();
  const { settings } = useSettings();
  const [pending, setPending] = useState(null);

  const doCopy = async (v) => {
    try {
      const r = await api.envCopy(v.scope, v.name);
      toast.ok(r.clearAfter ? t('vars.copiedSecret', { seconds: r.clearAfter }) : t('common.copied'));
    } catch (e) {
      toast.error(errorText(t, e));
    }
  };

  const copy = (v) => {
    if (v.sensitive && settings.security.warnOnCopy) setPending(v);
    else doCopy(v);
  };

  const dialog = pending && (
    <ConfirmDialog
      title={t('vars.copySecretTitle')}
      confirmLabel={t('common.copy')}
      onCancel={() => setPending(null)}
      onConfirm={() => {
        const v = pending;
        setPending(null);
        doCopy(v);
      }}
    >
      <p className="muted">{t('vars.copySecretBody', { name: pending.name })}</p>
    </ConfirmDialog>
  );

  return { copy, dialog };
}

export async function copyPlainText(text) {
  await navigator.clipboard.writeText(text);
}
