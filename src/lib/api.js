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
  envPlanApply: (plan, context) => call('env_plan_apply', { plan, context: context || null }),

  // Profiller
  profilesList: (includeArchived) => call('profiles_list', { includeArchived: !!includeArchived }),
  profileGet: (id) => call('profile_get', { id }),
  profileCreate: (input) => call('profile_create', { input }),
  profileUpdate: (id, input, note) => call('profile_update', { id, input, note }),
  profileDuplicate: (id, name) => call('profile_duplicate', { id, name }),
  profileRename: (id, name) => call('profile_rename', { id, name }),
  profileSetStatus: (id, status) => call('profile_set_status', { id, status }),
  profileDelete: (id) => call('profile_delete', { id }),
  profileRevisions: (id) => call('profile_revisions', { id }),
  profileRevisionGet: (id, revisionId) => call('profile_revision_get', { id, revisionId }),
  profileRestoreRevision: (id, revisionId) => call('profile_restore_revision', { id, revisionId }),
  profileRevealVar: (id, name) => call('profile_reveal_var', { id, name }),
  profileValidate: (id) => call('profile_validate', { id }),
  profileActivationPlans: (id) => call('profile_activation_plans', { id }),
  profileDeactivationPlan: (id, scope) => call('profile_deactivation_plan', { id, scope }),
  profileForgetActivation: (id, scope) => call('profile_forget_activation', { id, scope }),
  profileActivations: (id) => call('profile_activations', { id }),
  sensitivitySet: (name, sensitive) => call('sensitivity_set', { name, sensitive }),
  sensitivityList: () => call('sensitivity_list'),
};
