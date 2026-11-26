import { lazy } from 'react';

// Gezinme tanımı. Sayfalar tembel yüklenir; yalnızca açılan sayfanın kodu indirilir.
export const NAV_GROUPS = [
  {
    labelKey: 'nav.groupOverview',
    items: [{ id: 'dashboard', icon: 'dashboard', labelKey: 'nav.dashboard' }],
  },
  {
    labelKey: 'nav.groupEnvironment',
    items: [{ id: 'variables', icon: 'variable', labelKey: 'nav.variables' }],
  },
  {
    labelKey: 'nav.groupSystem',
    items: [{ id: 'settings', icon: 'settings', labelKey: 'nav.settings' }],
  },
];

export const PAGES = {
  dashboard: lazy(() => import('../pages/Dashboard.jsx')),
  variables: lazy(() => import('../pages/variables/Variables.jsx')),
  settings: lazy(() => import('../pages/Settings.jsx')),
};

export const DEFAULT_PAGE = 'dashboard';
