import { lazy } from 'react';

// Gezinme tanımı. Sayfalar tembel yüklenir; yalnızca açılan sayfanın kodu indirilir.
export const NAV_GROUPS = [
  {
    labelKey: 'nav.groupOverview',
    items: [{ id: 'dashboard', icon: 'dashboard', labelKey: 'nav.dashboard' }],
  },
  {
    labelKey: 'nav.groupEnvironment',
    items: [
      { id: 'variables', icon: 'variable', labelKey: 'nav.variables' },
      { id: 'path', icon: 'path', labelKey: 'nav.path' },
      { id: 'profiles', icon: 'profile', labelKey: 'nav.profiles' },
      { id: 'projects', icon: 'project', labelKey: 'nav.projects' },
    ],
  },
  {
    labelKey: 'nav.groupWorkflows',
    items: [
      { id: 'analysis', icon: 'analysis', labelKey: 'nav.analysis' },
      { id: 'compare', icon: 'compare', labelKey: 'nav.compare' },
      { id: 'snapshots', icon: 'snapshot', labelKey: 'nav.snapshots' },
      { id: 'transfer', icon: 'importExport', labelKey: 'nav.transfer' },
    ],
  },
  {
    labelKey: 'nav.groupSystem',
    items: [{ id: 'settings', icon: 'settings', labelKey: 'nav.settings' }],
  },
];

export const PAGES = {
  dashboard: lazy(() => import('../pages/Dashboard.jsx')),
  variables: lazy(() => import('../pages/variables/Variables.jsx')),
  analysis: lazy(() => import('../pages/analysis/Analysis.jsx')),
  compare: lazy(() => import('../pages/compare/Compare.jsx')),
  path: lazy(() => import('../pages/path/PathManager.jsx')),
  profiles: lazy(() => import('../pages/profiles/Profiles.jsx')),
  projects: lazy(() => import('../pages/projects/Projects.jsx')),
  snapshots: lazy(() => import('../pages/snapshots/Snapshots.jsx')),
  transfer: lazy(() => import('../pages/transfer/Transfer.jsx')),
  settings: lazy(() => import('../pages/Settings.jsx')),
};

export const DEFAULT_PAGE = 'dashboard';
