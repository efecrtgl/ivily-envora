use crate::profiles::model::{ProfilePath, ProfileVar, VarInput};
use serde::{Deserialize, Serialize};

/// Kullanıcının açıkça tanımladığı özel başlatma komutu. Kabuk üzerinden **çalıştırılmaz**;
/// program ve argümanlar ayrı ayrı verilir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchConfig {
    /// `pwsh` | `powershell` | `cmd`; `None` ayarlardaki varsayılanı kullanır.
    pub shell: Option<String>,
    pub custom: Option<CustomCommand>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub identifier: String,
    pub description: String,
    pub directory: String,
    pub profile_id: Option<i64>,
    pub profile_name: Option<String>,
    pub tags: Vec<String>,
    pub launch: LaunchConfig,
    pub created_at: String,
    pub updated_at: String,
    pub vars: Vec<ProfileVar>,
    pub paths: Vec<ProfilePath>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub identifier: String,
    pub description: String,
    pub directory: String,
    pub directory_exists: bool,
    pub profile_id: Option<i64>,
    pub profile_name: Option<String>,
    pub tags: Vec<String>,
    pub var_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: String,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub description: String,
    pub directory: String,
    #[serde(default)]
    pub profile_id: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub launch: LaunchConfig,
    #[serde(default)]
    pub vars: Vec<VarInput>,
    #[serde(default)]
    pub paths: Vec<ProfilePath>,
}
