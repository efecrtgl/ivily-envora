use serde::{Deserialize, Serialize};

/// Ortam değişkeni kapsamı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Uygulamanın kendi işlem ortamı (başlatıldığı andaki kopya). Kalıcı değildir.
    Process,
    /// `HKCU\Environment`
    User,
    /// `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`
    Machine,
}

impl Scope {
    pub const ALL: [Scope; 3] = [Scope::Process, Scope::User, Scope::Machine];
    pub const PERSISTENT: [Scope; 2] = [Scope::User, Scope::Machine];

    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Process => "process",
            Scope::User => "user",
            Scope::Machine => "machine",
        }
    }

    pub fn is_persistent(self) -> bool {
        !matches!(self, Scope::Process)
    }
}

/// Değer türü. Kayıt defterinde `REG_SZ` ve `REG_EXPAND_SZ` ayrımı korunur;
/// `%VAR%` başvuruları yalnızca `Expand` türünde Windows tarafından genişletilir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueKind {
    String,
    Expand,
    /// Ortam anahtarında beklenmeyen bir kayıt türü (ör. REG_DWORD). Salt okunur gösterilir.
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub kind: ValueKind,
    pub scope: Scope,
}

impl EnvVar {
    pub fn new(scope: Scope, name: &str, value: &str, kind: ValueKind) -> Self {
        EnvVar {
            name: name.to_string(),
            value: value.to_string(),
            kind,
            scope,
        }
    }
}

/// Karşılaştırma anahtarı: Windows'ta ortam değişkeni adları büyük/küçük harfe duyarsızdır.
pub fn name_key(name: &str) -> String {
    name.to_uppercase()
}
