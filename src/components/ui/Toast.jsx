import { createContext, useCallback, useContext, useMemo, useRef, useState } from 'react';
import Icon from './Icon.jsx';

const ToastCtx = createContext(null);

const ICONS = { ok: 'check', error: 'alert', warn: 'alert', info: 'info' };

export function ToastProvider({ children }) {
  const [items, setItems] = useState([]);
  const seq = useRef(0);

  const dismiss = useCallback((id) => setItems((xs) => xs.filter((x) => x.id !== id)), []);

  const push = useCallback(
    (tone, message, timeout = 4500) => {
      const id = ++seq.current;
      setItems((xs) => [...xs.slice(-3), { id, tone, message }]);
      if (timeout) setTimeout(() => dismiss(id), timeout);
    },
    [dismiss],
  );

  const api = useMemo(
    () => ({
      ok: (m) => push('ok', m),
      error: (m) => push('error', m, 7000),
      warn: (m) => push('warn', m, 6000),
      info: (m) => push('info', m),
    }),
    [push],
  );

  return (
    <ToastCtx.Provider value={api}>
      {children}
      <div className="toasts" aria-live="polite">
        {items.map((x) => (
          <div key={x.id} className={`toast ${x.tone}`} role={x.tone === 'error' ? 'alert' : 'status'}>
            <Icon name={ICONS[x.tone]} />
            <div style={{ flex: 1 }}>{x.message}</div>
            <button className="btn ghost sm icon-only" onClick={() => dismiss(x.id)} aria-label="×">
              <Icon name="x" size={12} />
            </button>
          </div>
        ))}
      </div>
    </ToastCtx.Provider>
  );
}

export function useToast() {
  return useContext(ToastCtx);
}
