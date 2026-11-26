import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { I18nProvider, useT } from '../i18n/index.jsx';
import { SettingsProvider, useSettings } from './settings.jsx';
import { NAV_GROUPS, PAGES, DEFAULT_PAGE } from './nav.js';
import { NavContext } from './navigation.js';
import { ToastProvider } from '../components/ui/Toast.jsx';
import Icon from '../components/ui/Icon.jsx';
import { Loading } from '../components/ui/primitives.jsx';
import ErrorBoundary from './ErrorBoundary.jsx';
import logo from '../../assets/logo.svg';

function Sidebar({ page, onNavigate }) {
  const t = useT();
  const { settings, set } = useSettings();
  const theme = settings.appearance.theme;
  const nextTheme = theme === 'dark' ? 'light' : theme === 'light' ? 'system' : 'dark';
  return (
    <nav className="sidebar" aria-label={t('app.name')}>
      <div className="brand">
        <img src={logo} alt="" />
        <div>
          <div className="brand-name">{t('app.name')}</div>
          <div className="brand-sub">{t('app.tagline')}</div>
        </div>
      </div>
      {NAV_GROUPS.map((g) => (
        <div key={g.labelKey}>
          <div className="nav-group-label">{t(g.labelKey)}</div>
          {g.items.map((item) => (
            <button
              key={item.id}
              type="button"
              className="nav-item"
              aria-current={page === item.id ? 'page' : undefined}
              onClick={() => onNavigate(item.id)}
            >
              <Icon name={item.icon} />
              <span>{t(item.labelKey)}</span>
            </button>
          ))}
        </div>
      ))}
      <div className="sidebar-footer">
        <span>v{__APP_VERSION__}</span>
        <button
          type="button"
          className="btn ghost sm icon-only"
          title={`${t('theme.toggle')} (${t(`theme.${theme}`)})`}
          aria-label={t('theme.toggle')}
          onClick={() => set('appearance.theme', nextTheme).catch(() => {})}
        >
          <Icon name={theme === 'light' ? 'sun' : theme === 'dark' ? 'moon' : 'monitor'} size={14} />
        </button>
      </div>
    </nav>
  );
}

function Shell() {
  const t = useT();
  const { settings, meta } = useSettings();
  const [route, setRoute] = useState({ page: DEFAULT_PAGE, params: null });
  const navigate = useCallback((page, params = null) => setRoute({ page, params }), []);

  // Ayarlar yüklendiğinde başlangıç sayfasına bir kez geç.
  const started = useRef(false);
  useEffect(() => {
    if (!meta.loaded || started.current) return;
    started.current = true;
    const start = settings.general.startPage;
    if (PAGES[start]) setRoute({ page: start, params: null });
  }, [meta.loaded, settings.general.startPage]);
  const nav = useMemo(() => ({ ...route, navigate }), [route, navigate]);
  const Page = PAGES[route.page] || PAGES[DEFAULT_PAGE];

  return (
    <NavContext.Provider value={nav}>
      <div className="shell">
        <Sidebar page={route.page} onNavigate={navigate} />
        <main className="main">
          {meta.degraded && (
            <div className="banner" role="alert">
              <Icon name="alert" /> {t('settings.degraded')}
            </div>
          )}
          <ErrorBoundary key={route.page}>
            <Suspense fallback={<Loading label={t('common.loading')} />}>
              <Page params={route.params} />
            </Suspense>
          </ErrorBoundary>
        </main>
      </div>
    </NavContext.Provider>
  );
}

function WithLanguage({ children }) {
  const { settings } = useSettings();
  return <I18nProvider lang={settings.general.language}>{children}</I18nProvider>;
}

export default function App() {
  return (
    <SettingsProvider>
      <WithLanguage>
        <ToastProvider>
          <Shell />
        </ToastProvider>
      </WithLanguage>
    </SettingsProvider>
  );
}
