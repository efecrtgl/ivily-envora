//! Veritabanı yedeğinden geri yükleme.
//!
//! Çalışan bir veritabanının üzerine yazmak güvenli olmadığından geri yükleme iki adımlıdır:
//! 1. Seçilen yedek doğrulanır (bütünlük, şema sürümü) ve veri dizinine `restore-pending.db`
//!    olarak kopyalanır.
//! 2. Uygulama yeniden başlatıldığında, veritabanı açılmadan önce mevcut dosya
//!    `envora.db.bak-<zaman>` adıyla saklanır ve bekleyen yedek yerine konur.
//!
//! Mevcut veritabanı hiçbir zaman silinmez.

use super::migrations;
use crate::error::{AppError, AppResult};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub const PENDING: &str = "restore-pending.db";
pub const DB_FILE: &str = "envora.db";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub schema_version: i64,
    pub latest_version: i64,
    pub integrity_ok: bool,
    pub size_bytes: u64,
    pub profiles: Option<i64>,
    pub projects: Option<i64>,
    pub snapshots: Option<i64>,
    pub restorable: bool,
}

fn count(c: &Connection, table: &str) -> Option<i64> {
    // `table` yalnızca bu dosyadaki sabitlerden gelir.
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table],
            |r| r.get(0),
        )
        .ok()?;
    if !exists {
        return None;
    }
    c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .ok()
}

/// Bir yedek dosyasını salt okunur açar ve denetler.
pub fn inspect(path: &Path) -> AppResult<BackupInfo> {
    if !path.is_file() {
        return Err(AppError::validation(
            "backup_not_found",
            path.to_string_lossy().into_owned(),
        ));
    }
    let c = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| AppError::validation("invalid_backup", ""))?;
    let schema_version: i64 = c
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|_| AppError::validation("invalid_backup", ""))?;
    let integrity: String = c
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|_| AppError::validation("invalid_backup", ""))?;
    let latest = migrations::latest_version();
    let has_settings = count(&c, "settings").is_some();
    let info = BackupInfo {
        schema_version,
        latest_version: latest,
        integrity_ok: integrity == "ok",
        size_bytes: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        profiles: count(&c, "profiles"),
        projects: count(&c, "projects"),
        snapshots: count(&c, "snapshots"),
        restorable: integrity == "ok"
            && schema_version >= 1
            && schema_version <= latest
            && has_settings,
    };
    Ok(info)
}

/// Yedeği doğrular ve bir sonraki açılışta geri yüklenmek üzere hazırlar.
pub fn schedule(data_dir: &Path, backup: &Path) -> AppResult<BackupInfo> {
    let info = inspect(backup)?;
    if !info.restorable {
        return Err(AppError::validation("invalid_backup", ""));
    }
    std::fs::create_dir_all(data_dir)?;
    let pending = data_dir.join(PENDING);
    std::fs::copy(backup, &pending)?;
    // Kopyanın da sağlam olduğundan emin ol.
    if !inspect(&pending)?.restorable {
        let _ = std::fs::remove_file(&pending);
        return Err(AppError::validation("invalid_backup", ""));
    }
    Ok(info)
}

pub fn cancel(data_dir: &Path) -> AppResult<()> {
    let p = data_dir.join(PENDING);
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

pub fn is_pending(data_dir: &Path) -> bool {
    data_dir.join(PENDING).is_file()
}

/// Açılışta, veritabanı açılmadan önce çağrılır. Bekleyen geri yükleme yoksa `Ok(None)`.
/// Başarılıysa eski veritabanının saklandığı yolu döndürür.
pub fn apply_pending(data_dir: &Path) -> AppResult<Option<PathBuf>> {
    let pending = data_dir.join(PENDING);
    if !pending.is_file() {
        return Ok(None);
    }
    if !inspect(&pending)?.restorable {
        // Bozuk bekleyen dosya: mevcut veritabanına dokunmadan vazgeç.
        let _ = std::fs::rename(&pending, data_dir.join("restore-rejected.db"));
        return Err(AppError::validation("invalid_backup", ""));
    }
    let db = data_dir.join(DB_FILE);
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let saved = data_dir.join(format!("{DB_FILE}.bak-{stamp}"));
    if db.exists() {
        std::fs::rename(&db, &saved)?;
        for ext in ["-wal", "-shm"] {
            let side = data_dir.join(format!("{DB_FILE}{ext}"));
            if side.exists() {
                let _ =
                    std::fs::rename(&side, data_dir.join(format!("{DB_FILE}.bak-{stamp}{ext}")));
            }
        }
    }
    std::fs::rename(&pending, &db)?;
    Ok(Some(saved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn schedule_and_apply_restore() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        // Mevcut veritabanı
        {
            let db = Db::open(&data.join(DB_FILE)).unwrap();
            db.with(|c| {
                c.execute("INSERT INTO settings(key, value, updated_at) VALUES ('marker', 'current', 'now')", [])?;
                Ok(())
            })
            .unwrap();
        }
        // Yedek
        let backup = dir.path().join("backup.db");
        {
            let db = Db::open(&dir.path().join("src.db")).unwrap();
            db.with(|c| {
                c.execute("INSERT INTO settings(key, value, updated_at) VALUES ('marker', 'backup', 'now')", [])?;
                Ok(())
            })
            .unwrap();
            db.backup_to(&backup).unwrap();
        }
        let info = inspect(&backup).unwrap();
        assert!(info.restorable && info.integrity_ok);
        assert_eq!(info.profiles, Some(0));

        schedule(&data, &backup).unwrap();
        assert!(is_pending(&data));
        let saved = apply_pending(&data).unwrap().unwrap();
        assert!(saved.exists(), "eski veritabanı saklanmalı");
        let db = Db::open(&data.join(DB_FILE)).unwrap();
        let v: String = db
            .with(|c| {
                Ok(
                    c.query_row("SELECT value FROM settings WHERE key='marker'", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(v, "backup");
        assert!(!is_pending(&data));
        assert_eq!(apply_pending(&data).unwrap(), None);
    }

    #[test]
    fn rejects_garbage_and_newer_schema() {
        let dir = tempfile::tempdir().unwrap();
        let junk = dir.path().join("junk.db");
        std::fs::write(&junk, b"not a database at all").unwrap();
        assert!(schedule(dir.path(), &junk).is_err());
        let newer = dir.path().join("newer.db");
        {
            let c = Connection::open(&newer).unwrap();
            c.execute_batch("CREATE TABLE settings(key TEXT); PRAGMA user_version = 999;")
                .unwrap();
        }
        assert!(!inspect(&newer).unwrap().restorable);
        assert!(schedule(dir.path(), &newer).is_err());
        assert!(!is_pending(dir.path()));
    }
}
