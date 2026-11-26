//! İçe ve dışa aktarma.
//!
//! - Uygulama biçimi: sürümlü JSON belgesi (`format: "ivily-envora"`, `version: 1`).
//! - Değişken listeleri için ayrıca CSV ve `.env`; PATH için satır başına bir girdi içeren metin.
//!
//! İçe aktarılan içerik **asla çalıştırılmaz**: yalnızca ayrıştırılır, doğrulanır ve önizlenir.
//! Ayrıştırılmış değerler arka uçta geçici bir önbellekte tutulur; ön uca yalnızca adlar,
//! maskeli önizleme ve uyarılar gönderilir.

pub mod csv;
pub mod dotenv;
pub mod export;
pub mod import;

use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::pathmgr::PathMod;
use crate::profiles::model::VarAction;
use crate::projects::model::LaunchConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const FORMAT: &str = "ivily-envora";
pub const VERSION: u32 = 1;
/// İçe aktarılabilecek en büyük dosya.
pub const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocKind {
    Variables,
    Profile,
    Project,
    Snapshot,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub format: String,
    pub version: u32,
    pub kind: DocKind,
    #[serde(default)]
    pub exported_at: String,
    #[serde(default)]
    pub app_version: String,
    /// Belge düz metin hassas değer içeriyor mu?
    #[serde(default)]
    pub contains_secrets: bool,
    pub data: serde_json::Value,
}

/// Dışa aktarılan değişken. `excluded: true` ise değer kasıtlı olarak dahil edilmemiştir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VarDoc {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub kind: Option<ValueKind>,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub excluded: bool,
    #[serde(default = "default_action")]
    pub action: VarAction,
}

fn default_action() -> VarAction {
    VarAction::Set
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathDocEntry {
    #[serde(flatten)]
    pub path: PathMod,
    pub scope: Scope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDoc {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub vars: Vec<VarDoc>,
    #[serde(default)]
    pub paths: Vec<PathDocEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDoc {
    pub name: String,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub directory: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub launch: LaunchConfig,
    /// İlişkili profilin adı (içe aktarmada aynı adlı profil varsa bağlanır).
    #[serde(default)]
    pub profile_name: Option<String>,
    #[serde(default)]
    pub vars: Vec<VarDoc>,
    #[serde(default)]
    pub paths: Vec<PathDocEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItemDoc {
    pub scope: Scope,
    pub name: String,
    pub kind: ValueKind,
    pub present: bool,
    #[serde(default)]
    pub sensitive: bool,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub excluded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDoc {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub created_at: String,
    pub items: Vec<SnapshotItemDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathDoc {
    #[serde(default)]
    pub scope: Option<Scope>,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Json,
    Csv,
    Env,
    Text,
}

/// Uzantıdan biçim tahmini.
pub fn detect_format(path: &Path) -> FileFormat {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name == ".env" || name.starts_with(".env.") || name.ends_with(".env") {
        return FileFormat::Env;
    }
    match path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .as_deref()
    {
        Some("json") => FileFormat::Json,
        Some("csv") => FileFormat::Csv,
        Some("txt") => FileFormat::Text,
        _ => FileFormat::Env,
    }
}

/// Dosyayı boyut sınırıyla okur. UTF-8 (BOM'lu/BOM'suz) ve UTF-16 LE (BOM'lu) desteklenir.
pub fn read_text(path: &Path) -> AppResult<String> {
    let meta = std::fs::metadata(path)?;
    if !meta.is_file() {
        return Err(AppError::validation("invalid_path", ""));
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(AppError::validation(
            "file_too_large",
            format!("en fazla {} MB", MAX_FILE_BYTES / 1024 / 1024),
        ));
    }
    let bytes = std::fs::read(path)?;
    decode(&bytes)
}

pub fn decode(bytes: &[u8]) -> AppResult<String> {
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let words: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&words)
            .map_err(|_| AppError::validation("invalid_encoding", ""));
    }
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8(bytes.to_vec()).map_err(|_| AppError::validation("invalid_encoding", ""))
}

/// Dosyayı önce geçici bir dosyaya yazıp yeniden adlandırır (yarım dosya bırakmamak için).
pub fn write_file(path: &Path, content: &str) -> AppResult<()> {
    if !path.is_absolute() {
        return Err(AppError::validation("invalid_path", ""));
    }
    let tmp = path.with_extension("envora-tmp");
    std::fs::write(&tmp, content.as_bytes())?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_formats() {
        assert_eq!(detect_format(Path::new("C:\\a\\.env")), FileFormat::Env);
        assert_eq!(
            detect_format(Path::new("C:\\a\\.env.local")),
            FileFormat::Env
        );
        assert_eq!(detect_format(Path::new("C:\\a\\prod.env")), FileFormat::Env);
        assert_eq!(detect_format(Path::new("C:\\a\\x.JSON")), FileFormat::Json);
        assert_eq!(detect_format(Path::new("C:\\a\\x.csv")), FileFormat::Csv);
        assert_eq!(
            detect_format(Path::new("C:\\a\\path.txt")),
            FileFormat::Text
        );
    }

    #[test]
    fn decodes_utf8_bom_and_utf16() {
        assert_eq!(decode(b"\xEF\xBB\xBFA=1").unwrap(), "A=1");
        let utf16: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain("Ş=1".encode_utf16().flat_map(|w| w.to_le_bytes()))
            .collect();
        assert_eq!(decode(&utf16).unwrap(), "Ş=1");
        assert!(decode(&[0xC3, 0x28]).is_err());
    }

    #[test]
    fn size_limit_and_atomic_write() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("out.json");
        write_file(&p, "{}").unwrap();
        assert_eq!(read_text(&p).unwrap(), "{}");
        assert!(write_file(Path::new("relative.json"), "x").is_err());
        let big = dir.path().join("big.env");
        let f = std::fs::File::create(&big).unwrap();
        f.set_len(MAX_FILE_BYTES + 1).unwrap();
        assert!(read_text(&big).is_err());
    }
}
