//! Ortam değişkeni adı ve değeri doğrulaması.
//!
//! Hata kodları ön uçta `errors.<kod>` anahtarıyla çevrilir. Hata ayrıntıları değer içermez.

use crate::error::{AppError, AppResult};
use serde::Serialize;

/// Windows ortam bloğundaki tek bir değişkenin en fazla uzunluğu (karakter).
pub const MAX_VALUE_LEN: usize = 32_767;
/// Uygulamanın kabul ettiği en uzun ad. Windows teknik olarak daha uzununa izin verir,
/// ancak pratikte 255'i aşan adlar araçlarda sorun çıkarır.
pub const MAX_NAME_LEN: usize = 255;

/// Kesin hata: bu adla değişken oluşturulamaz.
pub fn validate_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::validation("invalid_name_empty", ""));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::validation(
            "invalid_name_length",
            format!("en fazla {MAX_NAME_LEN} karakter"),
        ));
    }
    if name.contains('=') {
        return Err(AppError::validation(
            "invalid_name_equals",
            name.to_string(),
        ));
    }
    if name.chars().any(|c| c == '\0' || c.is_control()) {
        return Err(AppError::validation("invalid_name_control", ""));
    }
    Ok(())
}

pub fn validate_value(value: &str) -> AppResult<()> {
    if value.contains('\0') {
        return Err(AppError::validation("invalid_value_nul", ""));
    }
    if value.encode_utf16().count() > MAX_VALUE_LEN {
        return Err(AppError::validation(
            "invalid_value_length",
            format!("en fazla {MAX_VALUE_LEN} karakter"),
        ));
    }
    Ok(())
}

/// Uyarı niteliğinde bulgular (değişken yine de kullanılabilir).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Issue {
    /// Adın başında/sonunda boşluk var.
    NameWhitespace,
    /// Ad ASCII harf, rakam ve `_` dışında karakter içeriyor; bazı kabuklar erişemez.
    NamePortability,
    /// Ad rakamla başlıyor; POSIX kabuklarda kullanılamaz.
    NameLeadingDigit,
    /// Değerin başında/sonunda boşluk var.
    ValueWhitespace,
    /// Değer boş.
    EmptyValue,
    /// Değer çok satırlı.
    Multiline,
    /// `%VAR%` başvurusu var ama tür REG_SZ; Windows genişletmez.
    UnexpandedReference,
    /// Değer 2047 karakteri aşıyor; eski araçlar kesebilir.
    LongValue,
}

pub fn has_percent_reference(value: &str) -> bool {
    let mut parts = value.split('%');
    parts.next();
    let rest: Vec<&str> = parts.collect();
    // "%A%" → ["", "A", ""] : en az bir kapalı çift ve boş olmayan ad
    rest.len() >= 2
        && rest
            .iter()
            .step_by(2)
            .take(rest.len() / 2)
            .any(|s| !s.is_empty() && !s.contains(' '))
}

pub fn lint(name: &str, value: &str, kind: super::model::ValueKind) -> Vec<Issue> {
    let mut out = Vec::new();
    if name.trim() != name {
        out.push(Issue::NameWhitespace);
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '(' | ')' | '.' | '-'))
    {
        out.push(Issue::NamePortability);
    }
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.push(Issue::NameLeadingDigit);
    }
    if value.is_empty() {
        out.push(Issue::EmptyValue);
    } else if value.trim() != value {
        out.push(Issue::ValueWhitespace);
    }
    if value.contains('\n') || value.contains('\r') {
        out.push(Issue::Multiline);
    }
    if kind == super::model::ValueKind::String && has_percent_reference(value) {
        out.push(Issue::UnexpandedReference);
    }
    if value.encode_utf16().count() > 2047 {
        out.push(Issue::LongValue);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::model::ValueKind;

    #[test]
    fn names() {
        assert!(validate_name("JAVA_HOME").is_ok());
        assert!(validate_name("ProgramFiles(x86)").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("A=B").is_err());
        assert!(validate_name("A\0").is_err());
        assert!(validate_name("A\nB").is_err());
        assert!(validate_name(&"X".repeat(256)).is_err());
    }

    #[test]
    fn values() {
        assert!(validate_value("").is_ok());
        assert!(validate_value("a\0b").is_err());
        assert!(validate_value(&"x".repeat(MAX_VALUE_LEN)).is_ok());
        assert!(validate_value(&"x".repeat(MAX_VALUE_LEN + 1)).is_err());
    }

    #[test]
    fn validation_errors_do_not_echo_values() {
        let secret = concat!("gh", "p_", "fakefakefakefakefakefake", "\0");
        let e = validate_value(secret).unwrap_err();
        assert!(!serde_json::to_string(&e).unwrap().contains("ghp_"));
    }

    #[test]
    fn lint_findings() {
        assert!(lint("JAVA_HOME", "C:\\jdk", ValueKind::String).is_empty());
        assert!(lint("X", " a", ValueKind::String).contains(&Issue::ValueWhitespace));
        assert!(lint("X", "", ValueKind::String).contains(&Issue::EmptyValue));
        assert!(lint("1X", "a", ValueKind::String).contains(&Issue::NameLeadingDigit));
        assert!(lint("X", "%USERPROFILE%\\bin", ValueKind::String)
            .contains(&Issue::UnexpandedReference));
        assert!(!lint("X", "%USERPROFILE%\\bin", ValueKind::Expand)
            .contains(&Issue::UnexpandedReference));
        assert!(!lint("X", "100% done", ValueKind::String).contains(&Issue::UnexpandedReference));
    }
}
