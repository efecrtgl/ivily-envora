import { useEffect, useRef } from 'react';
import { Button } from './primitives.jsx';
import { useT } from '../../i18n/index.jsx';

export function Modal({ title, children, footer, onClose, wide, xwide, labelledBy = 'modal-title' }) {
  const ref = useRef(null);

  useEffect(() => {
    const prev = document.activeElement;
    const el = ref.current;
    const first = el?.querySelector('input, select, textarea, button:not([data-close])');
    first?.focus();
    const onKey = (e) => {
      if (e.key === 'Escape' && onClose) onClose();
      if (e.key === 'Tab' && el) {
        const items = el.querySelectorAll('button, input, select, textarea, [tabindex]:not([tabindex="-1"])');
        if (!items.length) return;
        const a = items[0];
        const b = items[items.length - 1];
        if (e.shiftKey && document.activeElement === a) {
          e.preventDefault();
          b.focus();
        } else if (!e.shiftKey && document.activeElement === b) {
          e.preventDefault();
          a.focus();
        }
      }
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      prev?.focus?.();
    };
  }, [onClose]);

  return (
    <div className="modal-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose?.()}>
      <div ref={ref} className={`modal ${wide ? 'wide' : ''} ${xwide ? 'xwide' : ''}`} role="dialog" aria-modal="true" aria-labelledby={labelledBy}>
        <div className="modal-header">
          <h2 id={labelledBy}>{title}</h2>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-footer">{footer}</div>}
      </div>
    </div>
  );
}

/**
 * Onay penceresi. `danger` true ise onay düğmesi tehlikeli işlem olarak gösterilir.
 */
export function ConfirmDialog({ title, children, confirmLabel, onConfirm, onCancel, danger, busy, disabled, wide }) {
  const t = useT();
  return (
    <Modal
      title={title}
      onClose={busy ? undefined : onCancel}
      wide={wide}
      footer={
        <>
          <Button variant="ghost" onClick={onCancel} disabled={busy} data-close>
            {t('common.cancel')}
          </Button>
          <Button variant={danger ? 'danger' : 'primary'} onClick={onConfirm} disabled={busy || disabled}>
            {busy && <span className="spinner" style={{ width: 14, height: 14 }} />}
            {confirmLabel || t('common.confirm')}
          </Button>
        </>
      }
    >
      {children}
    </Modal>
  );
}
