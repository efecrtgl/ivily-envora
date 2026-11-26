//! Güncelleme sistemi.
//!
//! - Kaynak yalnızca resmi GitHub sürümleridir (`github.com/efecrtgl/ivily-envora/releases`).
//! - Güncelleme paketleri minisign ile imzalanır; `tauri-plugin-updater` imzayı `tauri.conf.json`
//!   içindeki genel anahtarla doğrulamadan **kurmaz**.
//! - Denetim yalnızca bilgi verir; indirme ve kurulum kullanıcı onayından sonra yapılır.
//! - Kanallar: `stable` → en son kararlı sürümün `latest.json` dosyası; `prerelease` → GitHub API ile
//!   ön sürümler dahil en yeni yayımlanmış sürümün `latest.json` dosyası.

use crate::error::{AppError, AppResult};
use serde::Serialize;

pub const REPO: &str = "efecrtgl/ivily-envora";
pub const STABLE_ENDPOINT: &str =
    "https://github.com/efecrtgl/ivily-envora/releases/latest/download/latest.json";
const DOWNLOAD_PREFIX: &str = "https://github.com/efecrtgl/ivily-envora/releases/download/";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
    pub channel: String,
}

/// Uç noktanın yalnızca bu deponun sürüm dosyalarını gösterdiğini doğrular.
pub fn is_official_asset(url: &str) -> bool {
    url == STABLE_ENDPOINT
        || (url.starts_with(DOWNLOAD_PREFIX)
            && url.ends_with("/latest.json")
            && !url.contains(".."))
}

/// GitHub API yanıtından (sürüm listesi) ön sürümler dahil en yeni yayımlanmış sürümün
/// `latest.json` adresini seçer. Taslaklar atlanır.
pub fn pick_prerelease_endpoint(releases: &serde_json::Value) -> Option<String> {
    releases
        .as_array()?
        .iter()
        .filter(|r| !r["draft"].as_bool().unwrap_or(true))
        .find_map(|r| {
            r["assets"].as_array()?.iter().find_map(|a| {
                let name = a["name"].as_str()?;
                let url = a["browser_download_url"].as_str()?;
                (name == "latest.json" && is_official_asset(url)).then(|| url.to_string())
            })
        })
}

pub async fn endpoint_for(channel: &str) -> AppResult<String> {
    if channel != "prerelease" {
        return Ok(STABLE_ENDPOINT.to_string());
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::builder()
        .user_agent(format!("IvilyEnvora/{}", crate::APP_VERSION))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::Platform(format!("http: {e}")))?;
    let releases: serde_json::Value = client
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases?per_page=15"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Platform(format!("GitHub: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Platform(format!("GitHub: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::Platform(format!("GitHub: {e}")))?;
    pick_prerelease_endpoint(&releases).ok_or_else(|| AppError::NotFound("latest.json".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_official_assets_are_accepted() {
        assert!(is_official_asset(STABLE_ENDPOINT));
        assert!(is_official_asset(
            "https://github.com/efecrtgl/ivily-envora/releases/download/v0.2.0-beta.1/latest.json"
        ));
        assert!(!is_official_asset(
            "https://github.com/someone/else/releases/download/v1/latest.json"
        ));
        assert!(!is_official_asset("https://evil.example/latest.json"));
        assert!(!is_official_asset(
            "https://github.com/efecrtgl/ivily-envora/releases/download/../../x/latest.json"
        ));
        assert!(!is_official_asset(
            "https://github.com/efecrtgl/ivily-envora/releases/download/v1/setup.exe"
        ));
    }

    #[test]
    fn picks_newest_published_release_including_prereleases() {
        let releases = json!([
            { "draft": true, "prerelease": true, "assets": [{ "name": "latest.json", "browser_download_url": "https://github.com/efecrtgl/ivily-envora/releases/download/v0.3.0/latest.json" }] },
            { "draft": false, "prerelease": true, "assets": [
                { "name": "x.exe", "browser_download_url": "https://github.com/efecrtgl/ivily-envora/releases/download/v0.2.0-beta.1/x.exe" },
                { "name": "latest.json", "browser_download_url": "https://github.com/efecrtgl/ivily-envora/releases/download/v0.2.0-beta.1/latest.json" }
            ] },
            { "draft": false, "prerelease": false, "assets": [{ "name": "latest.json", "browser_download_url": "https://github.com/efecrtgl/ivily-envora/releases/download/v0.1.0/latest.json" }] }
        ]);
        assert_eq!(
            pick_prerelease_endpoint(&releases).as_deref(),
            Some("https://github.com/efecrtgl/ivily-envora/releases/download/v0.2.0-beta.1/latest.json")
        );
        let foreign = json!([{ "draft": false, "assets": [{ "name": "latest.json", "browser_download_url": "https://evil.example/latest.json" }] }]);
        assert_eq!(pick_prerelease_endpoint(&foreign), None);
    }
}
