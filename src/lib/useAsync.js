import { useCallback, useEffect, useRef, useState } from 'react';

/**
 * Bir arka uç çağrısını yükleniyor/hata/veri durumlarıyla çalıştırır.
 * `deps` değiştiğinde yeniden yükler; `reload()` elle yeniler. Bileşen kaldırıldıktan sonra
 * gelen yanıtlar yok sayılır.
 */
export function useAsync(fn, deps = []) {
  const [state, setState] = useState({ loading: true, error: null, data: null });
  const alive = useRef(true);
  const seq = useRef(0);

  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);

  // eslint-disable-next-line react-hooks/exhaustive-deps
  const run = useCallback(fn, deps);

  const reload = useCallback(() => {
    const id = ++seq.current;
    setState((s) => ({ ...s, loading: true, error: null }));
    return Promise.resolve()
      .then(run)
      .then(
        (data) => alive.current && id === seq.current && setState({ loading: false, error: null, data }),
        (error) => alive.current && id === seq.current && setState((s) => ({ loading: false, error, data: s.data })),
      );
  }, [run]);

  useEffect(() => {
    reload();
  }, [reload]);

  return { ...state, reload, setData: (data) => setState((s) => ({ ...s, data })) };
}
