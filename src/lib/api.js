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

  // Projeler
  projectsList: () => call('projects_list'),
  projectGet: (id) => call('project_get', { id }),
  projectInspectDir: (path) => call('project_inspect_dir', { path }),
  projectCreate: (input) => call('project_create', { input }),
  projectUpdate: (id, input) => call('project_update', { id, input }),
  projectDelete: (id) => call('project_delete', { id }),
  projectRevealVar: (id, name) => call('project_reveal_var', { id, name }),
  projectLaunchPreview: (id) => call('project_launch_preview', { id }),
  projectLaunch: (id, target, shell) => call('project_launch', { id, target, shell }),

  // Anlık görüntüler ve yedek
  snapshotsList: () => call('snapshots_list'),
  snapshotGet: (id) => call('snapshot_get', { id }),
  snapshotCreate: (request) => call('snapshot_create', { request }),
  snapshotUpdate: (id, name, description) => call('snapshot_update', { id, name, description }),
  snapshotDelete: (id) => call('snapshot_delete', { id }),
  snapshotReveal: (id, scope, name) => call('snapshot_reveal', { id, scope, name }),
  snapshotRestorePlans: (id, names) => call('snapshot_restore_plans', { id, names }),
  dbBackupInspect: (path) => call('db_backup_inspect', { path }),
  dbRestoreSchedule: (path) => call('db_restore_schedule', { path }),
  dbRestoreCancel: () => call('db_restore_cancel'),
  dbRestorePending: () => call('db_restore_pending'),
  appRestart: () => call('app_restart'),
  diagnosticsReport: () => call('diagnostics_report'),
  iepPreview: (method) => call('iep_preview', { method }),

  // İçe / dışa aktarma
  importPreview: (path, format) => call('import_preview', { path, format: format || null }),
  importDiscard: (token) => call('import_discard', { token }),
  importVariablesPlan: (token, scope, names, policy) => call('import_variables_plan', { token, scope, names, policy }),
  importPathPlan: (token, scope, mode) => call('import_path_plan', { token, scope, mode }),
  importCreate: (request) => call('import_create', { request }),
  exportWrite: (request, path, includeSensitive) => call('export_write', { request, path, includeSensitive }),

  // Analiz
  analysisRules: () => call('analysis_rules'),
  analysisRun: () => call('analysis_run'),
  analysisLast: () => call('analysis_last'),

  // Araçlar ve şablonlar
  discoveryRun: () => call('discovery_run'),
  discoveryLast: () => call('discovery_last'),
  templatesList: () => call('templates_list'),
  templateGet: (id) => call('template_get', { id }),
  templateCreate: (input) => call('template_create', { input }),
  templateUpdate: (id, input) => call('template_update', { id, input }),
  templateDuplicate: (id, name) => call('template_duplicate', { id, name }),
  templateDelete: (id) => call('template_delete', { id }),

  // Etkinlik
  activityList: (query) => call('activity_list', { query: query || {} }),
  activityClear: () => call('activity_clear'),
  compareSources: (left, right) => call('compare_sources', { left, right }),
  pathAnalyze: (scope) => call('path_analyze', { scope }),
  pathConflicts: () => call('path_conflicts'),
  sensitivitySet: (name, sensitive) => call('sensitivity_set', { name, sensitive }),
  sensitivityList: () => call('sensitivity_list'),
};
