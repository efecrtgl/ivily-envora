//! Profil veri modeli.

use crate::env::{Scope, ValueKind};
use crate::pathmgr::PathMod;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarAction {
    /// Değeri ayarla.
    Set,
    /// Değişkeni kaldır.
    Unset,
}

/// Profil değişkeni. `value`, ön uca gönderilirken hassas değişkenlerde `None` olur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileVar {
    pub name: String,
    pub value: Option<String>,
    pub sensitive: bool,
    /// `None`: kalıcı uygulamada mevcut tür korunur / içerikten çıkarılır.
    pub kind: Option<ValueKind>,
    /// Tercih edilen kapsam. `Process`: yalnızca proje başlatmada kullanılır, kalıcı yazılmaz.
    pub scope: Scope,
    pub action: VarAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePath {
    #[serde(flatten)]
    pub path: PathMod,
    pub scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Archived,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub status: Status,
    pub version: i64,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
    pub vars: Vec<ProfileVar>,
    pub paths: Vec<ProfilePath>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub status: Status,
    pub version: i64,
    pub var_count: i64,
    pub path_count: i64,
    pub has_secrets: bool,
    pub updated_at: String,
    /// Etkin (geri alınmamış) kalıcı uygulamaların kapsamları.
    pub active_scopes: Vec<Scope>,
}

/// Kaydetme girdisi. Hassas bir değişkenin değeri değişmediyse ön uç `keep: true` gönderir;
/// böylece gizli değer arayüze hiç gelmeden profil düzenlenebilir.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VarInput {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub keep: bool,
    /// Önceki adı (yeniden adlandırılan bir değişkenin saklı değerini korumak için).
    #[serde(default)]
    pub previous_name: Option<String>,
    /// `None`: otomatik sınıflandırma.
    #[serde(default)]
    pub sensitive: Option<bool>,
    #[serde(default)]
    pub kind: Option<ValueKind>,
    pub scope: Scope,
    pub action: VarAction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub vars: Vec<VarInput>,
    #[serde(default)]
    pub paths: Vec<ProfilePath>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub id: i64,
    pub version: i64,
    pub created_at: String,
    pub note: Option<String>,
    pub var_count: i64,
}
