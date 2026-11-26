//! Uygulama durumu ve veri dizini.

use crate::db::Db;
use crate::env::EnvProvider;
use crate::error::{AppError, AppResult};
use crate::settings::{self, Settings};
use parking_lot::RwLock;
use std::path::PathBuf;

pub struct AppState {
    pub db: Db,
    pub provider: Box<dyn EnvProvider>,
    pub settings: RwLock<Settings>,
    pub data_dir: PathBuf,
    /// Veritabanı açılamadıysa uygulama bellek içi veritabanıyla açılır ve bu alan dolar.
    /// Arayüz bu durumda kalıcı verinin kaydedilmeyeceğini belirten bir uyarı gösterir.
    pub degraded: Option<String>,
    /// Okuma sırasında varsayılana döndürülen ayar alanları (arayüzde bilgi olarak gösterilir).
    pub settings_fixed: Vec<String>,
    /// Açılışta uygulanan veritabanı geri yüklemesinin sonucu: `ok:<eski dosya>` veya `error:<kod>`.
    pub restore_note: Option<String>,
    /// Ayrıştırılmış içe aktarmalar (yalnızca bellekte, süreli).
    pub imports: crate::io::import::ImportCache,
    /// Son analiz sonucu (yalnızca bellekte).
    pub last_analysis: parking_lot::Mutex<Option<crate::analysis::Report>>,
}

/// `%LOCALAPPDATA%\IvilyEnvora`
pub fn data_dir() -> AppResult<PathBuf> {
    dirs::data_local_dir()
        .map(|d| d.join("IvilyEnvora"))
        .ok_or_else(|| AppError::Platform("yerel uygulama veri dizini bulunamadı".into()))
}

impl AppState {
    pub fn init() -> AppState {
        let dir = data_dir().unwrap_or_else(|_| std::env::temp_dir().join("IvilyEnvora"));
        let restore_note = match crate::db::restore::apply_pending(&dir) {
            Ok(Some(saved)) => Some(format!("ok:{}", saved.display())),
            Ok(None) => None,
            Err(e) => Some(format!("error:{}", e.code())),
        };
        let (db, degraded) = match Db::open(&dir.join("envora.db")) {
            Ok(db) => (db, None),
            Err(e) => (
                Db::open_in_memory().expect("bellek içi veritabanı açılamadı"),
                Some(e.to_string()),
            ),
        };
        let (settings, fixed) =
            settings::load(&db).unwrap_or_else(|_| (Settings::default(), vec!["*".into()]));
        if settings.activity.enabled {
            let _ = crate::activity::prune(&db, settings.activity.retention_days);
        }
        AppState {
            db,
            provider: Box::new(crate::env::windows::WindowsProvider::new()),
            settings: RwLock::new(settings),
            data_dir: dir,
            degraded,
            settings_fixed: fixed,
            restore_note,
            imports: Default::default(),
            last_analysis: Default::default(),
        }
    }

    #[cfg(test)]
    pub fn for_tests() -> AppState {
        AppState {
            db: Db::open_in_memory().unwrap(),
            provider: Box::new(crate::env::MemoryProvider::new()),
            settings: RwLock::new(Settings::default()),
            data_dir: std::env::temp_dir(),
            degraded: None,
            settings_fixed: Vec::new(),
            restore_note: None,
            imports: Default::default(),
            last_analysis: Default::default(),
        }
    }
}
