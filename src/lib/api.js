// Arka uca giden tüm çağrıların tek kapısı.
// Bileşenler doğrudan `invoke` kullanmaz; böylece komut yüzeyi tek dosyada görülebilir ve testlerde taklit edilebilir.
import { invoke } from '@tauri-apps/api/core';

export function inTauri() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function call(cmd, args) {
  if (!inTauri()) {
    throw { code: 'not_in_tauri', message: cmd };
  }
  return invoke(cmd, args);
}

export const api = {
  appInfo: () => call('app_info'),

  // Ayarlar ve veri
  settingsGet: () => call('settings_get'),
  settingsUpdate: (settings) => call('settings_update', { settings }),
  settingsReset: () => call('settings_reset'),
  dbStatus: () => call('db_status'),
  dbBackup: (dest) => call('db_backup', { dest }),

  // Ortam inceleme
  envList: (scopes) => call('env_list', { scopes: scopes || null }),
  envOverview: () => call('env_overview'),
  envDetail: (name) => call('env_detail', { name }),
  envReveal: (scope, name) => call('env_reveal', { scope, name }),
  envCopy: (scope, name) => call('env_copy', { scope, name }),
  envPlanPreview: (plan) => call('env_plan_preview', { plan }),
  envPlanApply: (plan) => call('env_plan_apply', { plan }),
  sensitivitySet: (name, sensitive) => call('sensitivity_set', { name, sensitive }),
  sensitivityList: () => call('sensitivity_list'),
};
