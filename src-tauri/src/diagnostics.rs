//! Tanılama raporu: hata bildirimlerine eklenebilecek, **değer ve ad içermeyen** özet.

use crate::db::Db;
use crate::env::provider::EnvProvider;
use crate::env::Scope;
use crate::error::AppResult;
use crate::settings::Settings;

fn count(db: &Db, table: &str) -> i64 {
    // `table` yalnızca bu dosyadaki sabitlerden gelir.
    db.with(|c| Ok(c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?))
        .unwrap_or(-1)
}

pub fn windows_version() -> String {
    imp::windows_version()
}

pub struct Extra {
    pub webview: String,
    pub analysis: Option<std::collections::BTreeMap<String, usize>>,
}

pub fn report(
    db: &Db,
    p: &dyn EnvProvider,
    settings: &Settings,
    extra: &Extra,
) -> AppResult<String> {
    let status = db.status()?;
    let mut lines = vec![
        "Ivily Envora — tanılama raporu (değişken adları ve değerleri içermez)".to_string(),
        String::new(),
        format!("Uygulama: {} {}", crate::APP_NAME, crate::APP_VERSION),
        format!("Windows: {}", windows_version()),
        format!("Mimari: {}", std::env::consts::ARCH),
        format!("WebView2: {}", extra.webview),
        format!(
            "Yönetici olarak çalışıyor: {}",
            if p.can_write_machine() {
                "evet"
            } else {
                "hayır"
            }
        ),
        String::new(),
        format!(
            "Veritabanı: şema v{}/v{}, bütünlük {}, boyut {} KB",
            status.schema_version,
            status.latest_version,
            if status.integrity_ok {
                "sağlam"
            } else {
                "SORUNLU"
            },
            status.size_bytes / 1024
        ),
    ];
    for scope in Scope::ALL {
        let n = p.list(scope).map(|v| v.len()).unwrap_or(0);
        lines.push(format!("Değişken sayısı ({}): {n}", scope.as_str()));
    }
    lines.push(format!(
        "Profiller: {} · Projeler: {} · Anlık görüntüler: {} · Şablonlar: {} · Etkinlik kaydı: {}",
        count(db, "profiles"),
        count(db, "projects"),
        count(db, "snapshots"),
        count(db, "templates"),
        count(db, "activity")
    ));
    lines.push(format!(
        "Ayarlar: dil={}, tema={}, otomatik yedek={}, sürüm sorgusu={}, güncelleme kanalı={}, IEP={}",
        settings.general.language,
        settings.appearance.theme,
        settings.backup.auto_snapshot_before_changes,
        settings.performance.probe_tool_versions,
        settings.updates.channel,
        settings.ecosystem.iep_enabled
    ));
    if let Some(a) = &extra.analysis {
        lines.push(format!("Son analiz: {a:?}"));
    }
    Ok(lines.join("\n"))
}

#[cfg(windows)]
mod imp {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    pub fn windows_version() -> String {
        let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", KEY_READ)
        else {
            return "bilinmiyor".into();
        };
        let get = |n: &str| k.get_value::<String, _>(n).unwrap_or_default();
        let build = get("CurrentBuild");
        let ubr: u32 = k.get_value("UBR").unwrap_or(0);
        // ProductName Windows 11'de de "Windows 10" der; derleme numarasıyla ayırt edilir.
        let product = if build.parse::<u32>().unwrap_or(0) >= 22000 {
            get("ProductName").replace("Windows 10", "Windows 11")
        } else {
            get("ProductName")
        };
        format!(
            "{product} {} (derleme {build}.{ubr})",
            get("DisplayVersion")
        )
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn windows_version() -> String {
        std::env::consts::OS.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::env::ValueKind;

    #[test]
    fn report_contains_no_names_or_values() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(
            Scope::User,
            "MY_PRIVATE_SERVICE_TOKEN",
            "tok-very-secret",
            ValueKind::String,
        );
        let r = report(
            &db,
            &p,
            &Settings::default(),
            &Extra {
                webview: "1.0".into(),
                analysis: None,
            },
        )
        .unwrap();
        assert!(!r.contains("tok-very-secret"));
        assert!(!r.contains("MY_PRIVATE_SERVICE_TOKEN"));
        assert!(r.contains("Değişken sayısı (user): 1"));
    }
}
