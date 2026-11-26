import { useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { api } from '../../lib/api.js';
import { useToast } from '../../components/ui/Toast.jsx';
import { ConfirmDialog } from '../../components/ui/Modal.jsx';
import { Callout, Field, Segmented } from '../../components/ui/primitives.jsx';

const SHELL_LABEL = { pwsh: 'PowerShell 7', powershell: 'Windows PowerShell', cmd: 'cmd' };

/**
 * Başlatma onayı. Ne çalıştırılacağı (kabuk ya da tam program yolu ve argümanlar), hangi dizinde
 * ve kaç ortam farkıyla çalışacağı açıkça gösterilir. Hiçbir proje betiği otomatik çalıştırılmaz.
 */
export default function LaunchDialog({ project, preview, mode, onClose }) {
  const t = useT();
  const toast = useToast();
  const initial = preview.shells.includes(preview.defaultShell) ? preview.defaultShell : preview.shells[0];
  const [shell, setShell] = useState(initial);
  const [busy, setBusy] = useState(false);
  const custom = mode === 'custom';
  const blocked = custom ? !!preview.customError || !preview.customProgram : !shell;

  const launch = async () => {
    setBusy(true);
    try {
      await api.projectLaunch(project.id, custom ? 'custom' : 'shell', custom ? null : shell);
      toast.ok(t('projects.launched'));
      onClose();
    } catch (e) {
      toast.error(errorText(t, e));
      setBusy(false);
    }
  };

  return (
    <ConfirmDialog
      title={custom ? t('projects.runCustom') : t('projects.launchTerminal')}
      confirmLabel={t('projects.launch')}
      onCancel={onClose}
      onConfirm={launch}
      busy={busy}
      disabled={blocked}
    >
      <div className="stack">
        {custom ? (
          preview.customError ? (
            <Callout tone="danger">{t(`errors.${preview.customError}`)}</Callout>
          ) : (
            <Field label={t('projects.willRun')}>
              <div className="card mono small selectable break" style={{ padding: 12 }}>
                <div>{preview.customProgram}</div>
                {preview.customArgs.map((a, i) => (
                  <div key={i} className="muted">
                    [{i}] {a}
                  </div>
                ))}
              </div>
            </Field>
          )
        ) : preview.shells.length ? (
          <Field label={t('projects.shell')}>
            <Segmented label={t('projects.shell')} value={shell} onChange={setShell} options={preview.shells.map((s) => ({ value: s, label: SHELL_LABEL[s] }))} />
          </Field>
        ) : (
          <Callout tone="danger">{t('errors.shell_not_found')}</Callout>
        )}
        <dl className="kv">
          <dt>{t('projects.workingDir')}</dt>
          <dd className="mono small break">{project.directory}</dd>
          <dt>{t('projects.envChanges')}</dt>
          <dd>{t('projects.envChangesCount', { count: preview.changes.length })}</dd>
        </dl>
        <Callout tone="info">{custom ? t('projects.customNote') : t('projects.shellNote')}</Callout>
      </div>
    </ConfirmDialog>
  );
}
