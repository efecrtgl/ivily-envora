import { useState } from 'react';
import { useT } from '../../i18n/index.jsx';
import Icon from '../ui/Icon.jsx';

/** Kapsamların farkını ve yeniden başlatma gerekliliğini açıklayan katlanabilir bilgi kutusu. */
export default function ScopeHelp({ defaultOpen = false }) {
  const t = useT();
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="callout info" style={{ display: 'block' }}>
      <button type="button" className="row" style={{ border: 0, background: 'none', padding: 0, cursor: 'pointer', width: '100%' }} onClick={() => setOpen(!open)} aria-expanded={open}>
        <Icon name="info" />
        <strong>{t('scope.helpTitle')}</strong>
        <span className="spacer" />
        <Icon name={open ? 'chevronDown' : 'chevronRight'} size={14} />
      </button>
      {open && (
        <div className="stack tight" style={{ marginTop: 10 }}>
          <p>
            <strong>{t('scope.process')}:</strong> {t('scope.processHelp')}
          </p>
          <p>
            <strong>{t('scope.user')}:</strong> {t('scope.userHelp')}
          </p>
          <p>
            <strong>{t('scope.machine')}:</strong> {t('scope.machineHelp')}
          </p>
          <p>{t('scope.restartNote')}</p>
        </div>
      )}
    </div>
  );
}
