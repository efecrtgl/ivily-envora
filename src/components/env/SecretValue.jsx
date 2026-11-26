import { useEffect, useState } from 'react';
import { useT, errorText } from '../../i18n/index.jsx';
import { useSettings } from '../../app/settings.jsx';
import { useToast } from '../ui/Toast.jsx';
import { api } from '../../lib/api.js';
import Icon from '../ui/Icon.jsx';

const MASK = '••••••••••••';

/**
 * Bir değişken değerini gösterir. Hassas değerler maskeli gelir (değer ön uçta yoktur);
 * "Göster" düğmesi arka uçtan değeri ister, ayarlanan süre sonunda yeniden gizler.
 * Gösterilen değer yalnızca bu bileşenin durumunda tutulur.
 */
export default function SecretValue({ v, multiline = false, reveal: revealFn }) {
  const t = useT();
  const toast = useToast();
  const { settings } = useSettings();
  const [revealed, setRevealed] = useState(null);
  const autoHide = settings.security.autoHideSeconds;

  useEffect(() => {
    if (revealed == null || !autoHide) return undefined;
    const id = setTimeout(() => setRevealed(null), autoHide * 1000);
    return () => clearTimeout(id);
  }, [revealed, autoHide]);

  // Başka bir değişkene geçildiğinde gösterilen değeri unut.
  useEffect(() => setRevealed(null), [v.name, v.scope]);

  if (v.kind === 'other') return <span className="faint">{t('vars.unsupportedType')}</span>;

  if (!v.masked) {
    if (v.value === '') return <span className="faint">{t('vars.emptyValue')}</span>;
    return <span className={`mono selectable ${multiline ? 'break' : 'truncate'}`} title={multiline ? undefined : v.value}>{v.value}</span>;
  }

  const reveal = async (e) => {
    e.stopPropagation();
    if (revealed != null) {
      setRevealed(null);
      return;
    }
    try {
      setRevealed(await (revealFn ? revealFn() : api.envReveal(v.scope, v.name)));
    } catch (err) {
      toast.error(errorText(t, err));
    }
  };

  if (revealFn === null) {
    return (
      <span className="mask" aria-label={t('vars.masked')}>
        {MASK}
      </span>
    );
  }

  return (
    <span className="secret">
      {revealed == null ? (
        <span className="mask" aria-label={t('vars.masked')}>{MASK}</span>
      ) : (
        <span className={`mono selectable ${multiline ? 'break' : 'truncate'}`}>{revealed}</span>
      )}
      <button
        type="button"
        className="btn ghost sm icon-only"
        onClick={reveal}
        title={revealed == null ? t('common.reveal') : t('common.hide')}
        aria-label={revealed == null ? t('common.reveal') : t('common.hide')}
      >
        <Icon name={revealed == null ? 'eye' : 'eyeOff'} size={14} />
      </button>
    </span>
  );
}
