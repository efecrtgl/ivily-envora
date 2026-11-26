//! SQLite veritabanı: bağlantı, geçişler, bütünlük denetimi ve yedekleme.

pub mod migrations;

use crate::error::{AppError, AppResult};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

pub struct Db {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    pub schema_version: i64,
    pub latest_version: i64,
    pub integrity_ok: bool,
    pub size_bytes: u64,
}

impl Db {
    /// Dosya veritabanını açar ve bekleyen geçişleri uygular.
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        let db = Db {
            conn: Mutex::new(conn),
            path: Some(path.to_path_buf()),
        };
        db.configure(true)?;
        db.migrate()?;
        Ok(db)
    }

    /// Testler için bellek içi veritabanı.
    pub fn open_in_memory() -> AppResult<Self> {
        let db = Db {
            conn: Mutex::new(Connection::open_in_memory()?),
            path: None,
        };
        db.configure(false)?;
        db.migrate()?;
        Ok(db)
    }

    fn configure(&self, file: bool) -> AppResult<()> {
        let c = self.conn.lock();
        c.pragma_update(None, "foreign_keys", "ON")?;
        c.busy_timeout(std::time::Duration::from_secs(5))?;
        if file {
            c.pragma_update(None, "journal_mode", "WAL")?;
            c.pragma_update(None, "synchronous", "NORMAL")?;
            // Silinen sayfaların içeriği dosyada kalmasın (hassas değerler şifreli olsa da).
            c.pragma_update(None, "secure_delete", "ON")?;
        }
        Ok(())
    }

    pub fn schema_version(&self) -> AppResult<i64> {
        Ok(self
            .conn
            .lock()
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }

    fn migrate(&self) -> AppResult<()> {
        let mut c = self.conn.lock();
        let current: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
        let latest = migrations::latest_version();
        if current > latest {
            // Daha yeni bir sürümün veritabanı: veriyi bozmamak için dokunmadan reddet.
            return Err(AppError::Database(format!(
                "veritabanı şeması ({current}) bu uygulama sürümünden ({latest}) daha yeni"
            )));
        }
        for m in migrations::MIGRATIONS
            .iter()
            .filter(|m| m.version > current)
        {
            let tx = c.transaction()?;
            tx.execute_batch(m.sql).map_err(|e| {
                AppError::Database(format!("geçiş {} ({}) başarısız: {e}", m.version, m.name))
            })?;
            tx.pragma_update(None, "user_version", m.version)?;
            tx.commit()?;
        }
        Ok(())
    }

    /// Bağlantıyı kilitleyip kapanışı çalıştırır.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let c = self.conn.lock();
        f(&c)
    }

    /// Kapanışı tek bir işlem içinde çalıştırır; hata olursa tüm değişiklikler geri alınır.
    pub fn tx<T>(&self, f: impl FnOnce(&rusqlite::Transaction) -> AppResult<T>) -> AppResult<T> {
        let mut c = self.conn.lock();
        let tx = c.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }

    pub fn integrity_check(&self) -> AppResult<bool> {
        let res: String = self
            .conn
            .lock()
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        Ok(res == "ok")
    }

    /// Tutarlı bir kopya oluşturur (`VACUUM INTO`). Hedef dosya önceden var olmamalıdır.
    pub fn backup_to(&self, dest: &Path) -> AppResult<()> {
        if dest.exists() {
            return Err(AppError::Conflict("yedek dosyası zaten var".into()));
        }
        let dest_str = dest
            .to_str()
            .ok_or_else(|| AppError::validation("invalid_path", "yol UTF-8 değil"))?;
        self.conn.lock().execute("VACUUM INTO ?1", [dest_str])?;
        Ok(())
    }

    pub fn status(&self) -> AppResult<DbStatus> {
        let size_bytes = self
            .path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);
        Ok(DbStatus {
            schema_version: self.schema_version()?,
            latest_version: migrations::latest_version(),
            integrity_ok: self.integrity_check()?,
            size_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_fresh_database_to_latest() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(db.schema_version().unwrap(), migrations::latest_version());
        assert!(db.integrity_check().unwrap());
    }

    #[test]
    fn reopening_is_idempotent_and_keeps_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        {
            let db = Db::open(&path).unwrap();
            db.with(|c| {
                c.execute(
                    "INSERT INTO settings(key, value, updated_at) VALUES ('k', '1', 'now')",
                    [],
                )?;
                Ok(())
            })
            .unwrap();
        }
        let db = Db::open(&path).unwrap();
        let v: String = db
            .with(
                |c| Ok(c.query_row("SELECT value FROM settings WHERE key='k'", [], |r| r.get(0))?),
            )
            .unwrap();
        assert_eq!(v, "1");
        assert_eq!(db.schema_version().unwrap(), migrations::latest_version());
    }

    #[test]
    fn refuses_newer_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        {
            let c = Connection::open(&path).unwrap();
            c.pragma_update(None, "user_version", 999).unwrap();
        }
        assert!(Db::open(&path).is_err());
    }

    #[test]
    fn transaction_rolls_back_on_error() {
        let db = Db::open_in_memory().unwrap();
        let r: AppResult<()> = db.tx(|tx| {
            tx.execute(
                "INSERT INTO settings(key, value, updated_at) VALUES ('a', '1', 'now')",
                [],
            )?;
            Err(AppError::Internal("boom".into()))
        });
        assert!(r.is_err());
        let n: i64 = db
            .with(|c| Ok(c.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn backup_creates_valid_copy() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("a.db")).unwrap();
        let dest = dir.path().join("b.db");
        db.backup_to(&dest).unwrap();
        let copy = Db::open(&dest).unwrap();
        assert!(copy.integrity_check().unwrap());
        assert!(
            db.backup_to(&dest).is_err(),
            "var olan dosyanın üzerine yazmamalı"
        );
    }
}
