import { lazy } from 'react';

// Gezinme tanımı. Sayfalar tembel yüklenir; yalnızca açılan sayfanın kodu indirilir.
export const NAV_GROUPS = [
  {
    labelKey: 'nav.groupOverview',
    items: [{ id: 'dashboard', icon: 'dashboard', labelKey: 'nav.dashboard' }],
  },
  {
    labelKey: 'nav.groupSystem',
    items: [{ id: 'settings', icon: 'settings', labelKey: 'nav.settings' }],
  },
];

export const PAGES = {
  dashboard: lazy(() => import('../pages/Dashboard.jsx')),
  settings: lazy(() => import('../pages/Settings.jsx')),
};

export const DEFAULT_PAGE = 'dashboard';
