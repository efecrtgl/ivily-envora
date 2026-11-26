//! Uygulama genelindeki hata türü.
//!
//! Hata mesajları **hiçbir zaman** ortam değişkeni değeri içermez. Yalnızca değişken
//! adları, kapsamlar ve işlem türleri gibi hassas olmayan bağlam bilgisi taşınır.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Kullanıcı girdisi doğrulamadan geçemedi. `code` ön uçta çevrilir.
    #[error("doğrulama hatası: {code}")]
    Validation { code: &'static str, detail: String },

    #[error("bulunamadı: {0}")]
    NotFound(String),

    #[error("yetki gerekli: {0}")]
    PermissionDenied(String),

    #[error("işlem kullanıcı tarafından iptal edildi")]
    Cancelled,

    #[error("çakışma: {0}")]
    Conflict(String),

    #[error("veritabanı hatası: {0}")]
    Database(String),

    #[error("güvenli depolama hatası: {0}")]
    Vault(String),

    #[error("G/Ç hatası: {0}")]
    Io(String),

    #[error("platform hatası: {0}")]
    Platform(String),

    #[error("desteklenmiyor: {0}")]
    Unsupported(String),

    #[error("iç hata: {0}")]
    Internal(String),
}

impl AppError {
    pub fn validation(code: &'static str, detail: impl Into<String>) -> Self {
        AppError::Validation {
            code,
            detail: detail.into(),
        }
    }

    /// Ön uçta i18n anahtarına dönüştürülen kararlı hata kodu.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Validation { code, .. } => code,
            AppError::NotFound(_) => "not_found",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::Cancelled => "cancelled",
            AppError::Conflict(_) => "conflict",
            AppError::Database(_) => "database",
            AppError::Vault(_) => "vault",
            AppError::Io(_) => "io",
            AppError::Platform(_) => "platform",
            AppError::Unsupported(_) => "unsupported",
            AppError::Internal(_) => "internal",
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            AppError::PermissionDenied(e.to_string())
        } else {
            AppError::Io(e.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        // serde_json hata metni girdiden parça içerebilir; yalnızca konum bilgisini koru.
        AppError::validation(
            "invalid_json",
            format!("satır {}, sütun {}", e.line(), e.column()),
        )
    }
}

#[derive(Serialize)]
struct WireError<'a> {
    code: &'a str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let message = match self {
            AppError::Validation { detail, .. } => detail.clone(),
            other => other.to_string(),
        };
        WireError {
            code: self.code(),
            message,
        }
        .serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_error_does_not_echo_input() {
        let secret = concat!("{\"k\": gh", "p_fakevalue}");
        let err: AppError = serde_json::from_str::<serde_json::Value>(secret)
            .unwrap_err()
            .into();
        let wire = serde_json::to_string(&err).unwrap();
        assert!(
            !wire.contains("ghp_"),
            "hata mesajı girdi değerini içermemeli: {wire}"
        );
        assert!(wire.contains("invalid_json"));
    }
}
