//! Dışa aktarma. Hassas değerler varsayılan olarak **dahil edilmez**; dahil edilirse dosyada
//! düz metin olarak bulunur ve belge `containsSecrets: true` ile işaretlenir.

use super::*;
use crate::db::Db;
use crate::env::inspect::{view_of, Overrides};
use crate::env::model::name_key;
use crate::env::provider::EnvProvider;
use crate::profiles::model::ProfileVar;
use crate::profiles::store as profiles;
use crate::projects::store as projects;
use crate::snapshots::store as snapshots;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub count: usize,
    pub excluded_sensitive: usize,
    pub contains_secrets: bool,
}

pub struct Output {
    pub content: String,
    pub result: ExportResult,
}

fn document(kind: DocKind, contains_secrets: bool, data: serde_json::Value) -> String {
    let doc = Document {
        format: FORMAT.into(),
        version: VERSION,
        kind,
        exported_at: chrono::Utc::now().to_rfc3339(),
        app_version: crate::APP_VERSION.into(),
        contains_secrets,
        data,
    };
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

fn var_docs(vars: &[ProfileVar], include_sensitive: bool) -> (Vec<VarDoc>, usize) {
    let mut excluded = 0;
    let docs = vars
        .iter()
        .map(|v| {
            let drop = v.sensitive && !include_sensitive;
            if drop {
                excluded += 1;
            }
            VarDoc {
                name: v.name.clone(),
                value: if drop { None } else { v.value.clone() },
                scope: Some(v.scope),
                kind: v.kind,
                sensitive: v.sensitive,
                excluded: drop,
                action: v.action,
            }
        })
        .collect();
    (docs, excluded)
}

fn path_docs(paths: &[crate::profiles::model::ProfilePath]) -> Vec<PathDocEntry> {
    paths
        .iter()
        .map(|p| PathDocEntry {
            path: p.path.clone(),
            scope: p.scope,
        })
        .collect()
}

pub fn variables(
    p: &dyn EnvProvider,
    overrides: &Overrides,
    scopes: &[Scope],
    names: Option<&[String]>,
    format: FileFormat,
    include_sensitive: bool,
) -> AppResult<Output> {
    let wanted: Option<HashSet<String>> = names.map(|n| n.iter().map(|x| name_key(x)).collect());
    let mut vars = Vec::new();
    let mut excluded = 0;
    for &scope in scopes {
        for v in p.list(scope)? {
            if v.kind == ValueKind::Other
                || wanted
                    .as_ref()
                    .is_some_and(|w| !w.contains(&name_key(&v.name)))
            {
                continue;
            }
            let sensitive = view_of(&v, overrides).sensitive;
            if sensitive && !include_sensitive {
                excluded += 1;
                // JSON'da ad korunur (değer yok); CSV/.env'de satır tamamen çıkarılır.
                if format == FileFormat::Json {
                    vars.push(VarDoc {
                        name: v.name,
                        value: None,
                        scope: Some(scope),
                        kind: Some(v.kind),
                        sensitive,
                        excluded: true,
                        action: VarAction::Set,
                    });
                }
                continue;
            }
            vars.push(VarDoc {
                name: v.name,
                value: Some(v.value),
                scope: Some(scope),
                kind: Some(v.kind),
                sensitive,
                excluded: false,
                action: VarAction::Set,
            });
        }
    }
    let contains_secrets = vars.iter().any(|v| v.sensitive && v.value.is_some());
    let count = vars.iter().filter(|v| !v.excluded).count();
    let content = match format {
        FileFormat::Json => document(
            DocKind::Variables,
            contains_secrets,
            serde_json::json!({ "variables": vars }),
        ),
        FileFormat::Csv => {
            let rows: Vec<Vec<String>> = vars
                .iter()
                .map(|v| {
                    vec![
                        v.name.clone(),
                        v.value.clone().unwrap_or_default(),
                        v.scope.map(|s| s.as_str().to_string()).unwrap_or_default(),
                        v.kind.map(|k| k.as_str().to_string()).unwrap_or_default(),
                    ]
                })
                .collect();
            csv::write(&["name", "value", "scope", "kind"], &rows)
        }
        FileFormat::Env => {
            let entries: Vec<(String, String)> = vars
                .iter()
                .map(|v| (v.name.clone(), v.value.clone().unwrap_or_default()))
                .collect();
            let header = format!(
                "Ivily Envora {} — {}\nBu dosya hiçbir komut içermez; değerler olduğu gibi yazılmıştır.",
                crate::APP_VERSION,
                chrono::Utc::now().to_rfc3339()
            );
            dotenv::write(&entries, &header)
        }
        FileFormat::Text => return Err(AppError::validation("unsupported_format", "text")),
    };
    Ok(Output {
        content,
        result: ExportResult {
            count,
            excluded_sensitive: excluded,
            contains_secrets,
        },
    })
}

pub fn profile(db: &Db, id: i64, include_sensitive: bool) -> AppResult<Output> {
    let f = profiles::load_full(db, id)?;
    let (vars, excluded) = var_docs(&f.vars, include_sensitive);
    let contains_secrets = vars.iter().any(|v| v.sensitive && v.value.is_some());
    let doc = ProfileDoc {
        name: f.name,
        description: f.description,
        category: f.category,
        tags: f.tags,
        vars,
        paths: path_docs(&f.paths),
    };
    let count = doc.vars.len();
    Ok(Output {
        content: document(
            DocKind::Profile,
            contains_secrets,
            serde_json::to_value(doc).unwrap_or_default(),
        ),
        result: ExportResult {
            count,
            excluded_sensitive: excluded,
            contains_secrets,
        },
    })
}

pub fn project(db: &Db, id: i64, include_sensitive: bool) -> AppResult<Output> {
    let pr = projects::load_full(db, id)?;
    let (vars, excluded) = var_docs(&pr.vars, include_sensitive);
    let contains_secrets = vars.iter().any(|v| v.sensitive && v.value.is_some());
    let doc = ProjectDoc {
        name: pr.name,
        identifier: Some(pr.identifier),
        description: pr.description,
        directory: pr.directory,
        tags: pr.tags,
        launch: pr.launch,
        profile_name: pr.profile_name,
        vars,
        paths: path_docs(&pr.paths),
    };
    let count = doc.vars.len();
    Ok(Output {
        content: document(
            DocKind::Project,
            contains_secrets,
            serde_json::to_value(doc).unwrap_or_default(),
        ),
        result: ExportResult {
            count,
            excluded_sensitive: excluded,
            contains_secrets,
        },
    })
}

pub fn snapshot(db: &Db, id: i64, include_sensitive: bool) -> AppResult<Output> {
    let meta = snapshots::get_meta(db, id)?;
    let mut excluded = 0;
    let items: Vec<SnapshotItemDoc> = snapshots::items(db, id)?
        .into_iter()
        .map(|i| {
            let drop = i.sensitive && !include_sensitive && i.value.is_some();
            if drop {
                excluded += 1;
            }
            SnapshotItemDoc {
                scope: i.scope,
                name: i.name,
                kind: i.kind,
                present: i.present,
                sensitive: i.sensitive,
                value: if drop { None } else { i.value },
                excluded: drop,
            }
        })
        .collect();
    let contains_secrets = items.iter().any(|i| i.sensitive && i.value.is_some());
    let count = items.len();
    let doc = SnapshotDoc {
        name: meta.name,
        description: meta.description,
        source: meta.source,
        created_at: meta.created_at,
        items,
    };
    Ok(Output {
        content: document(
            DocKind::Snapshot,
            contains_secrets,
            serde_json::to_value(doc).unwrap_or_default(),
        ),
        result: ExportResult {
            count,
            excluded_sensitive: excluded,
            contains_secrets,
        },
    })
}

pub fn path(p: &dyn EnvProvider, scope: Scope, format: FileFormat) -> AppResult<Output> {
    let value = p.get(scope, "Path")?.map(|v| v.value).unwrap_or_default();
    let entries: Vec<String> = crate::pathmgr::entries::split(&value)
        .into_iter()
        .filter(|e| !e.trim().is_empty())
        .collect();
    let count = entries.len();
    let content = match format {
        FileFormat::Text => {
            let mut s = entries.join("\r\n");
            s.push_str("\r\n");
            s
        }
        FileFormat::Json => document(
            DocKind::Path,
            false,
            serde_json::to_value(PathDoc {
                scope: Some(scope),
                entries,
            })
            .unwrap_or_default(),
        ),
        _ => return Err(AppError::validation("unsupported_format", "")),
    };
    Ok(Output {
        content,
        result: ExportResult {
            count,
            excluded_sensitive: 0,
            contains_secrets: false,
        },
    })
}

pub fn template(db: &Db, id: &str, en: bool) -> AppResult<Output> {
    let t = crate::templates::get(db, id, en)?;
    let input = crate::templates::input_of(&t);
    let count = input.vars.len();
    Ok(Output {
        content: document(
            DocKind::Template,
            false,
            serde_json::to_value(input).unwrap_or_default(),
        ),
        result: ExportResult {
            count,
            excluded_sensitive: 0,
            contains_secrets: false,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;

    fn provider() -> MemoryProvider {
        let p = MemoryProvider::new();
        p.seed(Scope::User, "EDITOR", "code", ValueKind::String);
        p.seed(
            Scope::User,
            "GITHUB_TOKEN",
            "ghp-like-secret-value",
            ValueKind::String,
        );
        p.seed(Scope::User, "Path", "C:\\a;;C:\\b", ValueKind::Expand);
        p
    }

    #[test]
    fn excludes_sensitive_by_default_in_every_format() {
        let p = provider();
        let o = Overrides::new();
        for f in [FileFormat::Json, FileFormat::Csv, FileFormat::Env] {
            let out = variables(&p, &o, &[Scope::User], None, f, false).unwrap();
            assert!(
                !out.content.contains("ghp-like-secret-value"),
                "{f:?} gizli değer içermemeli"
            );
            assert_eq!(out.result.excluded_sensitive, 1);
            assert!(!out.result.contains_secrets);
        }
        let json = variables(&p, &o, &[Scope::User], None, FileFormat::Json, false)
            .unwrap()
            .content;
        assert!(
            json.contains("\"excluded\": true"),
            "JSON'da hariç tutulan ad işaretlenir"
        );
    }

    #[test]
    fn includes_sensitive_only_when_asked() {
        let p = provider();
        let out = variables(
            &p,
            &Overrides::new(),
            &[Scope::User],
            None,
            FileFormat::Json,
            true,
        )
        .unwrap();
        assert!(out.content.contains("ghp-like-secret-value"));
        assert!(out.result.contains_secrets);
        assert!(out.content.contains("\"containsSecrets\": true"));
    }

    #[test]
    fn path_export_skips_empty_entries() {
        let p = provider();
        let out = path(&p, Scope::User, FileFormat::Text).unwrap();
        assert_eq!(out.content, "C:\\a\r\nC:\\b\r\n");
        assert_eq!(out.result.count, 2);
    }
}
