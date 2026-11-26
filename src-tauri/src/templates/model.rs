use crate::env::{Scope, ValueKind};
use crate::profiles::model::ProfilePath;
use serde::{Deserialize, Serialize};

/// Şablon değişkeninin rolü: öneri ile zorunlu yapılandırmayı açıkça ayırır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Kullanıcının doldurması gereken değer.
    Required,
    /// Önerilen ayar.
    Recommended,
    /// İsteğe bağlı ayar.
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateVar {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub kind: Option<ValueKind>,
    pub scope: Scope,
    pub role: Role,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    /// `builtin:<ad>` veya `user:<kimlik>`
    pub id: String,
    pub builtin: bool,
    pub name: String,
    pub description: String,
    pub category: String,
    pub vars: Vec<TemplateVar>,
    pub paths: Vec<ProfilePath>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub vars: Vec<TemplateVar>,
    #[serde(default)]
    pub paths: Vec<ProfilePath>,
}
