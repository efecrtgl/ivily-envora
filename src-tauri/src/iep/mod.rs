//! Ivily Ecosystem Protocol (IEP) — sürüm 1, yalnızca yerel.
//!
//! Envora, Ivily Center'a bağımlı değildir ve sürekli çalışan bir hizmet açmaz. Bir ekosistem
//! uygulaması Envora'yı `ivily-envora.exe --iep` ile başlatır, standart girdiye **tek** bir JSON
//! istek yazar ve standart çıktıdan tek bir JSON yanıt okur; süreç ardından kapanır.
//!
//! - Varsayılan olarak kapalıdır; açıldığında her yetenek kullanıcı tarafından ayrıca izin verilmelidir.
//! - Yanıtlar yalnızca özet içerir (sayılar, profil/proje adları). Ortam değişkeni **değerleri ve
//!   adları hiçbir zaman** paylaşılmaz.
//! - İstekler sürüm, yöntem ve boyut açısından doğrulanır; bilinmeyen alanlar yok sayılmaz, reddedilir.
//! - Ayrıntılar: `docs/IEP.md`.

use crate::db::Db;
use crate::env::provider::EnvProvider;
use crate::env::Scope;
use crate::error::AppResult;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const PROTOCOL_VERSION: u32 = 1;
pub const APP_ID: &str = "com.efecrtgl.ivilyenvora";
pub const CLI_FLAG: &str = "--iep";
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// Yetenekler ve karşılık gelen yöntemler.
pub static CAPABILITIES: &[(&str, &str)] = &[
    ("health", "envora.health"),
    ("profiles.summary", "envora.profiles.summary"),
    ("path.summary", "envora.path.summary"),
    ("analysis.summary", "envora.analysis.summary"),
    ("activity.summary", "envora.activity.summary"),
    ("workspace.projects", "envora.workspace.projects"),
    ("update.metadata", "envora.update.metadata"),
];

pub fn known_capability(id: &str) -> bool {
    CAPABILITIES.iter().any(|(c, _)| *c == id)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub iep: u32,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub iep: u32,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
}

fn err(id: Value, code: &'static str, message: impl Into<String>) -> Response {
    Response {
        iep: PROTOCOL_VERSION,
        id,
        result: None,
        error: Some(ErrorBody {
            code,
            message: message.into(),
        }),
    }
}

fn ok(id: Value, result: Value) -> Response {
    Response {
        iep: PROTOCOL_VERSION,
        id,
        result: Some(result),
        error: None,
    }
}

pub struct Context<'a> {
    pub db: &'a Db,
    pub provider: &'a dyn EnvProvider,
    pub settings: &'a Settings,
}

fn identity(ctx: &Context) -> Value {
    json!({
        "appId": APP_ID,
        "name": crate::APP_NAME,
        "version": crate::APP_VERSION,
        "protocolVersions": [PROTOCOL_VERSION],
        "enabled": ctx.settings.ecosystem.iep_enabled,
        "capabilities": CAPABILITIES.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
        "granted": granted(ctx),
    })
}

fn granted(ctx: &Context) -> Vec<String> {
    if !ctx.settings.ecosystem.iep_enabled {
        return Vec::new();
    }
    ctx.settings
        .ecosystem
        .capabilities
        .iter()
        .filter(|c| known_capability(c))
        .cloned()
        .collect()
}

/// Yöntemin yanıtını üretir (izin denetimi `handle` içinde yapılır).
pub fn summary(ctx: &Context, method: &str) -> AppResult<Value> {
    Ok(match method {
        "envora.health" => {
            let s = ctx.db.status()?;
            json!({ "status": if s.integrity_ok { "ok" } else { "degraded" }, "schemaVersion": s.schema_version, "integrity": s.integrity_ok })
        }
        "envora.profiles.summary" => {
            let list = crate::profiles::store::list(ctx.db, false)?;
            json!(list
                .into_iter()
                .map(|p| json!({ "name": p.name, "variables": p.var_count, "pathChanges": p.path_count, "activeScopes": p.active_scopes, "status": p.status }))
                .collect::<Vec<_>>())
        }
        "envora.path.summary" => {
            let mut out = Vec::new();
            for scope in Scope::PERSISTENT {
                let r = crate::pathmgr::analyze::analyze(ctx.provider, scope)?;
                out.push(json!({ "scope": scope, "entries": r.entries.len(), "length": r.length, "issues": r.counts }));
            }
            json!(out)
        }
        "envora.analysis.summary" => {
            let o = crate::security::overrides::all(ctx.db)?;
            let cfg = crate::analysis::Config {
                disabled: ctx.settings.analysis.disabled_rules.clone(),
                severity: ctx.settings.analysis.severity.clone(),
            };
            let r = crate::analysis::run(ctx.db, ctx.provider, &o, &cfg)?;
            let mut rules: BTreeMap<&str, usize> = BTreeMap::new();
            for f in &r.findings {
                *rules.entry(f.rule).or_default() += 1;
            }
            json!({ "counts": r.counts, "rules": rules, "generatedAt": r.generated_at })
        }
        "envora.activity.summary" => {
            let since = (chrono::Utc::now() - chrono::Duration::days(7)).to_rfc3339();
            let rows: Vec<(String, i64)> = ctx.db.with(|c| {
                let mut st = c.prepare(
                    "SELECT category, COUNT(*) FROM activity WHERE ts >= ?1 GROUP BY category",
                )?;
                let rows = st.query_map([since], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })?;
            json!({ "days": 7, "byCategory": rows.into_iter().collect::<BTreeMap<_, _>>() })
        }
        "envora.workspace.projects" => {
            let list = crate::projects::store::list(ctx.db)?;
            json!(list
                .into_iter()
                .map(|p| json!({ "name": p.name, "identifier": p.identifier, "directory": p.directory, "profile": p.profile_name }))
                .collect::<Vec<_>>())
        }
        "envora.update.metadata" => {
            json!({ "version": crate::APP_VERSION, "channel": ctx.settings.updates.channel })
        }
        _ => return Err(crate::error::AppError::NotFound(method.to_string())),
    })
}

/// Ham isteği işler ve yanıtı döndürür. Hiçbir zaman panik yapmaz; her hata bir yanıttır.
pub fn handle(raw: &str, ctx: &Context) -> Response {
    if raw.len() > MAX_REQUEST_BYTES {
        return err(Value::Null, "request_too_large", "");
    }
    let req: Request = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(_) => {
            return err(
                Value::Null,
                "invalid_request",
                "geçersiz JSON veya bilinmeyen alan",
            )
        }
    };
    let id = match &req.id {
        Value::String(_) | Value::Number(_) => req.id.clone(),
        _ => return err(Value::Null, "invalid_request", "id metin veya sayı olmalı"),
    };
    if req.iep != PROTOCOL_VERSION {
        return err(
            id,
            "unsupported_version",
            format!("desteklenen: {PROTOCOL_VERSION}"),
        );
    }
    if req
        .params
        .as_ref()
        .is_some_and(|p| !p.is_null() && !p.as_object().is_some_and(|o| o.is_empty()))
    {
        return err(id, "invalid_params", "bu sürümde yöntemler parametre almaz");
    }
    if req.method == "iep.hello" {
        return ok(id, identity(ctx));
    }
    let Some((cap, _)) = CAPABILITIES.iter().find(|(_, m)| *m == req.method) else {
        return err(id, "unknown_method", req.method);
    };
    if !ctx.settings.ecosystem.iep_enabled {
        return err(id, "disabled", "IEP Envora ayarlarında kapalı");
    }
    if !granted(ctx).iter().any(|g| g == cap) {
        return err(
            id,
            "permission_denied",
            format!("izin verilmemiş yetenek: {cap}"),
        );
    }
    match summary(ctx, &req.method) {
        Ok(v) => ok(id, v),
        Err(e) => err(id, "internal", e.code()),
    }
}

/// `main` tarafından çağrılır: `--iep` kipi değilse `None`.
pub fn maybe_run_cli() -> Option<i32> {
    if std::env::args().nth(1).as_deref() != Some(CLI_FLAG) {
        return None;
    }
    use std::io::{Read, Write};
    let mut input = String::new();
    let _ = std::io::stdin()
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_string(&mut input);
    let response = (|| -> AppResult<Response> {
        let dir = crate::state::data_dir()?;
        let db = Db::open(&dir.join("envora.db"))?;
        let (settings, _) = crate::settings::load(&db)?;
        let provider = crate::env::windows::WindowsProvider::new();
        let ctx = Context {
            db: &db,
            provider: &provider,
            settings: &settings,
        };
        let r = handle(&input, &ctx);
        if r.error.is_none() {
            crate::activity::record_quiet(
                &db,
                "ecosystem",
                "iep_request",
                None,
                crate::activity::Status::Ok,
                None,
            );
        }
        Ok(r)
    })()
    .unwrap_or_else(|e| err(Value::Null, "internal", e.code()));
    let text = serde_json::to_string(&response).unwrap_or_default();
    let mut out = std::io::stdout();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
    Some(if response.error.is_none() { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::env::ValueKind;

    fn settings(enabled: bool, caps: &[&str]) -> Settings {
        let mut s = Settings::default();
        s.ecosystem.iep_enabled = enabled;
        s.ecosystem.capabilities = caps.iter().map(|c| c.to_string()).collect();
        s
    }

    fn call(ctx: &Context, method: &str) -> Response {
        handle(&format!(r#"{{"iep":1,"id":"1","method":"{method}"}}"#), ctx)
    }

    #[test]
    fn hello_works_even_when_disabled_and_lists_no_grants() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let s = settings(false, &["health"]);
        let ctx = Context {
            db: &db,
            provider: &p,
            settings: &s,
        };
        let r = call(&ctx, "iep.hello");
        let v = r.result.unwrap();
        assert_eq!(v["appId"], APP_ID);
        assert_eq!(v["enabled"], false);
        assert_eq!(v["granted"], json!([]));
        assert_eq!(call(&ctx, "envora.health").error.unwrap().code, "disabled");
    }

    #[test]
    fn capabilities_are_enforced() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let s = settings(true, &["health"]);
        let ctx = Context {
            db: &db,
            provider: &p,
            settings: &s,
        };
        assert!(call(&ctx, "envora.health").result.is_some());
        assert_eq!(
            call(&ctx, "envora.profiles.summary").error.unwrap().code,
            "permission_denied"
        );
        assert_eq!(
            call(&ctx, "envora.shell.exec").error.unwrap().code,
            "unknown_method"
        );
    }

    #[test]
    fn validates_envelope() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let s = settings(true, &["health"]);
        let ctx = Context {
            db: &db,
            provider: &p,
            settings: &s,
        };
        assert_eq!(handle("nope", &ctx).error.unwrap().code, "invalid_request");
        assert_eq!(
            handle(r#"{"iep":2,"id":1,"method":"iep.hello"}"#, &ctx)
                .error
                .unwrap()
                .code,
            "unsupported_version"
        );
        assert_eq!(
            handle(r#"{"iep":1,"id":1,"method":"iep.hello","extra":1}"#, &ctx)
                .error
                .unwrap()
                .code,
            "invalid_request"
        );
        assert_eq!(
            handle(r#"{"iep":1,"id":{},"method":"iep.hello"}"#, &ctx)
                .error
                .unwrap()
                .code,
            "invalid_request"
        );
        assert_eq!(
            handle(
                r#"{"iep":1,"id":1,"method":"envora.health","params":{"x":1}}"#,
                &ctx
            )
            .error
            .unwrap()
            .code,
            "invalid_params"
        );
        assert_eq!(
            handle(&"x".repeat(MAX_REQUEST_BYTES + 1), &ctx)
                .error
                .unwrap()
                .code,
            "request_too_large"
        );
    }

    #[test]
    fn summaries_never_contain_variable_names_or_values() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "Path", "C:\\x", ValueKind::Expand);
        p.seed(
            Scope::Machine,
            "SUPER_SECRET_NAME_TOKEN",
            "value-should-not-leak",
            ValueKind::String,
        );
        let all: Vec<&str> = CAPABILITIES.iter().map(|(c, _)| *c).collect();
        let s = settings(true, &all);
        let ctx = Context {
            db: &db,
            provider: &p,
            settings: &s,
        };
        for (_, m) in CAPABILITIES {
            let r = call(&ctx, m);
            assert!(r.error.is_none(), "{m}: {:?}", r.error);
            let text = serde_json::to_string(&r).unwrap();
            assert!(!text.contains("value-should-not-leak"), "{m}");
            assert!(!text.contains("SUPER_SECRET_NAME_TOKEN"), "{m}");
        }
    }
}
