//! Doğrulanmış uygulama ayarları.
//!
//! Ayarlar `settings` tablosunda tek bir JSON belgesi olarak saklanır. Yükleme toleranslıdır:
//! eksik alanlar varsayılanla doldurulur, geçersiz alanlar **yalnızca o alan** için varsayılana
//! döndürülür ve düzeltilen alanlar raporlanır. Böylece sürüm yükseltmelerinde ayarlar sessizce
//! sıfırlanmaz.

use crate::db::Db;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const KEY: &str = "app";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub general: General,
    pub appearance: Appearance,
    pub dashboard: Dashboard,
    pub security: Security,
    pub backup: Backup,
    pub activity: Activity,
    pub notifications: Notifications,
    pub updates: Updates,
    pub performance: Performance,
    pub workspace: Workspace,
    pub ecosystem: Ecosystem,
    pub analysis: Analysis,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Analysis {
    /// Devre dışı bırakılan kural kimlikleri.
    pub disabled_rules: Vec<String>,
    /// Kural kimliği → `info` | `warning` | `error`
    pub severity: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct General {
    /// `tr` | `en`
    pub language: String,
    /// Açılışta gösterilecek sayfa kimliği.
    pub start_page: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Appearance {
    /// `system` | `dark` | `light`
    pub theme: String,
    /// `comfortable` | `compact`
    pub density: String,
    pub reduce_motion: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Dashboard {
    /// Görünür bölümlerin sıralı listesi.
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Security {
    /// Gösterilen hassas değerin otomatik olarak yeniden gizlenme süresi (saniye, 0 = kapalı).
    pub auto_hide_seconds: u32,
    /// Hassas değer panoya kopyalanmadan önce uyar.
    pub warn_on_copy: bool,
    /// Panoya kopyalanan hassas değerin temizlenme süresi (saniye, 0 = kapalı).
    pub clipboard_clear_seconds: u32,
    /// Dışa aktarmalarda hassas değişkenleri varsayılan olarak hariç tut.
    pub exclude_sensitive_on_export: bool,
    /// Anlık görüntülerde hassas değişkenleri varsayılan olarak hariç tut.
    pub exclude_sensitive_in_snapshots: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Backup {
    /// Kalıcı değişikliklerden önce otomatik anlık görüntü al.
    pub auto_snapshot_before_changes: bool,
    /// Saklanacak en fazla otomatik anlık görüntü sayısı.
    pub max_auto_snapshots: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Activity {
    pub enabled: bool,
    /// Etkinlik kayıtlarının saklanma süresi (gün).
    pub retention_days: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Notifications {
    pub show_success: bool,
    /// Kalıcı değişikliklerden sonra "açık uygulamaları yeniden başlatın" hatırlatması.
    pub restart_hint: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Updates {
    /// `stable` | `prerelease`
    pub channel: String,
    pub check_on_startup: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Performance {
    /// Araç keşfinde sürüm sorgusu (`--version`) çalıştırılsın mı.
    pub probe_tool_versions: bool,
    /// Sürüm sorgusu zaman aşımı (ms).
    pub probe_timeout_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Workspace {
    /// Proje terminali için varsayılan kabuk: `pwsh` | `powershell` | `cmd`
    pub default_shell: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Ecosystem {
    /// Ivily Ecosystem Protocol özetleri. Varsayılan olarak kapalı.
    pub iep_enabled: bool,
}

pub const DASHBOARD_SECTIONS: &[&str] = &[
    "overview",
    "pathHealth",
    "analysis",
    "security",
    "profiles",
    "projects",
    "tools",
    "activity",
];

impl Default for General {
    fn default() -> Self {
        General {
            language: "tr".into(),
            start_page: "dashboard".into(),
        }
    }
}
impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            theme: "system".into(),
            density: "comfortable".into(),
            reduce_motion: false,
        }
    }
}
impl Default for Dashboard {
    fn default() -> Self {
        Dashboard {
            sections: DASHBOARD_SECTIONS.iter().map(|s| s.to_string()).collect(),
        }
    }
}
impl Default for Security {
    fn default() -> Self {
        Security {
            auto_hide_seconds: 30,
            warn_on_copy: true,
            clipboard_clear_seconds: 30,
            exclude_sensitive_on_export: true,
            exclude_sensitive_in_snapshots: false,
        }
    }
}
impl Default for Backup {
    fn default() -> Self {
        Backup {
            auto_snapshot_before_changes: true,
            max_auto_snapshots: 50,
        }
    }
}
impl Default for Activity {
    fn default() -> Self {
        Activity {
            enabled: true,
            retention_days: 90,
        }
    }
}
impl Default for Notifications {
    fn default() -> Self {
        Notifications {
            show_success: true,
            restart_hint: true,
        }
    }
}
impl Default for Updates {
    fn default() -> Self {
        Updates {
            channel: "stable".into(),
            check_on_startup: true,
        }
    }
}
impl Default for Performance {
    fn default() -> Self {
        Performance {
            probe_tool_versions: true,
            probe_timeout_ms: 3000,
        }
    }
}
impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            default_shell: "powershell".into(),
        }
    }
}

fn one_of(
    field: &str,
    value: &mut String,
    allowed: &[&str],
    default: &str,
    fixed: &mut Vec<String>,
) {
    if !allowed.contains(&value.as_str()) {
        *value = default.to_string();
        fixed.push(field.to_string());
    }
}

fn clamp(field: &str, value: &mut u32, min: u32, max: u32, fixed: &mut Vec<String>) {
    if *value < min || *value > max {
        *value = (*value).clamp(min, max);
        fixed.push(field.to_string());
    }
}

impl Settings {
    /// Geçersiz alanları düzeltir ve düzeltilen alanların adlarını döndürür.
    pub fn normalize(&mut self) -> Vec<String> {
        let mut f = Vec::new();
        one_of(
            "general.language",
            &mut self.general.language,
            &["tr", "en"],
            "tr",
            &mut f,
        );
        one_of(
            "general.startPage",
            &mut self.general.start_page,
            &["dashboard", "variables", "path", "profiles", "projects"],
            "dashboard",
            &mut f,
        );
        one_of(
            "appearance.theme",
            &mut self.appearance.theme,
            &["system", "dark", "light"],
            "system",
            &mut f,
        );
        one_of(
            "appearance.density",
            &mut self.appearance.density,
            &["comfortable", "compact"],
            "comfortable",
            &mut f,
        );
        one_of(
            "updates.channel",
            &mut self.updates.channel,
            &["stable", "prerelease"],
            "stable",
            &mut f,
        );
        one_of(
            "workspace.defaultShell",
            &mut self.workspace.default_shell,
            &["pwsh", "powershell", "cmd"],
            "powershell",
            &mut f,
        );

        let before = self.dashboard.sections.clone();
        let mut seen = std::collections::HashSet::new();
        self.dashboard
            .sections
            .retain(|s| DASHBOARD_SECTIONS.contains(&s.as_str()) && seen.insert(s.clone()));
        if self.dashboard.sections != before {
            f.push("dashboard.sections".into());
        }

        let before = self.analysis.severity.len();
        self.analysis
            .severity
            .retain(|k, v| k.len() <= 80 && ["info", "warning", "error"].contains(&v.as_str()));
        if self.analysis.severity.len() != before {
            f.push("analysis.severity".into());
        }
        let before = self.analysis.disabled_rules.len();
        self.analysis
            .disabled_rules
            .retain(|k| !k.is_empty() && k.len() <= 80);
        self.analysis.disabled_rules.dedup();
        if self.analysis.disabled_rules.len() != before {
            f.push("analysis.disabledRules".into());
        }

        clamp(
            "security.autoHideSeconds",
            &mut self.security.auto_hide_seconds,
            0,
            3600,
            &mut f,
        );
        clamp(
            "security.clipboardClearSeconds",
            &mut self.security.clipboard_clear_seconds,
            0,
            600,
            &mut f,
        );
        clamp(
            "backup.maxAutoSnapshots",
            &mut self.backup.max_auto_snapshots,
            5,
            1000,
            &mut f,
        );
        clamp(
            "activity.retentionDays",
            &mut self.activity.retention_days,
            1,
            3650,
            &mut f,
        );
        clamp(
            "performance.probeTimeoutMs",
            &mut self.performance.probe_timeout_ms,
            500,
            15000,
            &mut f,
        );
        f
    }

    /// Ham JSON'u ayarlara dönüştürür. Bölüm düzeyinde tür hatası varsa yalnızca o bölüm
    /// varsayılana döner; diğer bölümler korunur.
    pub fn from_value_lenient(v: &Value) -> (Settings, Vec<String>) {
        let mut fixed = Vec::new();
        let mut s = Settings::default();
        let Some(obj) = v.as_object() else {
            return (s, vec!["*".into()]);
        };
        macro_rules! section {
            ($key:literal, $field:ident) => {
                if let Some(raw) = obj.get($key) {
                    match serde_json::from_value(raw.clone()) {
                        Ok(val) => s.$field = val,
                        Err(_) => fixed.push($key.to_string()),
                    }
                }
            };
        }
        section!("general", general);
        section!("appearance", appearance);
        section!("dashboard", dashboard);
        section!("security", security);
        section!("backup", backup);
        section!("activity", activity);
        section!("notifications", notifications);
        section!("updates", updates);
        section!("performance", performance);
        section!("workspace", workspace);
        section!("ecosystem", ecosystem);
        section!("analysis", analysis);
        fixed.extend(s.normalize());
        (s, fixed)
    }
}

/// Ayarları yükler. Kayıt yoksa varsayılanları döndürür (yazmaz).
pub fn load(db: &Db) -> AppResult<(Settings, Vec<String>)> {
    let raw: Option<String> = db.with(|c| {
        use rusqlite::OptionalExtension;
        Ok(
            c.query_row("SELECT value FROM settings WHERE key = ?1", [KEY], |r| {
                r.get(0)
            })
            .optional()?,
        )
    })?;
    match raw {
        None => Ok((Settings::default(), Vec::new())),
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(v) => Ok(Settings::from_value_lenient(&v)),
            Err(_) => Ok((Settings::default(), vec!["*".into()])),
        },
    }
}

/// Ayarları doğrular ve kaydeder. Geçersiz bir değer içeren güncelleme reddedilir
/// (sessizce düzeltilmez); düzeltme yalnızca okuma sırasında yapılır.
pub fn save(db: &Db, settings: &Settings) -> AppResult<()> {
    let mut check = settings.clone();
    let fixed = check.normalize();
    if !fixed.is_empty() {
        return Err(AppError::validation("invalid_setting", fixed.join(", ")));
    }
    let text = serde_json::to_string(settings).map_err(|e| AppError::Internal(e.to_string()))?;
    let now = chrono::Utc::now().to_rfc3339();
    db.with(|c| {
        c.execute(
            "INSERT INTO settings(key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            rusqlite::params![KEY, text, now],
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn defaults_are_valid() {
        let mut s = Settings::default();
        assert!(s.normalize().is_empty());
    }

    #[test]
    fn missing_fields_get_defaults_and_known_fields_are_kept() {
        // Eski bir sürümden kalma, yalnızca bazı alanları olan belge.
        let (s, fixed) = Settings::from_value_lenient(&json!({
            "general": { "language": "en" },
            "appearance": { "theme": "light" }
        }));
        assert!(fixed.is_empty());
        assert_eq!(s.general.language, "en");
        assert_eq!(s.appearance.theme, "light");
        assert_eq!(s.activity.retention_days, 90);
    }

    #[test]
    fn invalid_field_resets_only_that_field() {
        let (s, fixed) = Settings::from_value_lenient(&json!({
            "general": { "language": "de" },
            "appearance": { "theme": "dark" },
            "activity": { "retentionDays": 0 }
        }));
        assert_eq!(s.general.language, "tr");
        assert_eq!(s.appearance.theme, "dark");
        assert_eq!(s.activity.retention_days, 1);
        assert!(fixed.contains(&"general.language".to_string()));
        assert!(fixed.contains(&"activity.retentionDays".to_string()));
    }

    #[test]
    fn type_error_resets_only_that_section() {
        let (s, fixed) = Settings::from_value_lenient(&json!({
            "security": "bozuk",
            "appearance": { "theme": "light" }
        }));
        assert_eq!(s.security, Security::default());
        assert_eq!(s.appearance.theme, "light");
        assert_eq!(fixed, vec!["security".to_string()]);
    }

    #[test]
    fn unknown_dashboard_sections_are_removed() {
        let (s, _) = Settings::from_value_lenient(
            &json!({ "dashboard": { "sections": ["overview", "x", "overview"] } }),
        );
        assert_eq!(s.dashboard.sections, vec!["overview".to_string()]);
    }

    #[test]
    fn save_rejects_invalid_and_round_trips_valid() {
        let db = Db::open_in_memory().unwrap();
        let mut s = Settings::default();
        s.appearance.theme = "neon".into();
        assert!(save(&db, &s).is_err());

        s.appearance.theme = "light".into();
        s.general.language = "en".into();
        save(&db, &s).unwrap();
        let (loaded, fixed) = load(&db).unwrap();
        assert!(fixed.is_empty());
        assert_eq!(loaded, s);
    }
}
