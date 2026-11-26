//! Analiz kuralları. Her kural yalnızca okur ve bulgu üretir.
//! Bulgular hiçbir zaman değişken **değeri** içermez.

use super::{Context, Finding, Rule, Severity};
use crate::env::inspect::{expand_references, view_of};
use crate::env::model::name_key;
use crate::env::validate::{has_percent_reference, lint, Issue};
use crate::env::{effective, EnvVar, Scope, ValueKind};
use crate::pathmgr::analyze::{self as path_analyze, Flag};
use crate::profiles::validate::{is_relative_path, references};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

fn f(
    rule: &'static str,
    category: &'static str,
    target: impl Into<String>,
    scope: Option<Scope>,
) -> Finding {
    Finding {
        rule,
        category,
        severity: Severity::Info,
        target: target.into(),
        scope,
        params: BTreeMap::new(),
    }
}

fn with(mut x: Finding, k: &'static str, v: impl ToString) -> Finding {
    x.params.insert(k, v.to_string());
    x
}

// ---------- PATH ----------

fn path_flag(ctx: &Context, flag: Flag, rule: &'static str) -> Vec<Finding> {
    let mut out = Vec::new();
    for scope in Scope::PERSISTENT {
        if let Ok(r) = path_analyze::analyze(ctx.provider, scope) {
            for e in r.entries.iter().filter(|e| e.flags.contains(&flag)) {
                let target = if e.raw.trim().is_empty() {
                    format!("#{}", e.index + 1)
                } else {
                    e.raw.trim().to_string()
                };
                out.push(with(
                    f(rule, "path", target, Some(scope)),
                    "index",
                    e.index + 1,
                ));
            }
        }
    }
    out
}

fn path_missing(ctx: &Context) -> Vec<Finding> {
    let mut v = path_flag(ctx, Flag::Missing, "path.missing");
    v.extend(path_flag(ctx, Flag::NotDirectory, "path.missing"));
    v
}
fn path_duplicate(ctx: &Context) -> Vec<Finding> {
    path_flag(ctx, Flag::Duplicate, "path.duplicate")
}
fn path_relative(ctx: &Context) -> Vec<Finding> {
    path_flag(ctx, Flag::Relative, "path.relative")
}
fn path_empty(ctx: &Context) -> Vec<Finding> {
    path_flag(ctx, Flag::Empty, "path.empty")
}
fn path_unresolved(ctx: &Context) -> Vec<Finding> {
    path_flag(ctx, Flag::UnresolvedReference, "path.unresolved")
}

fn path_cross_scope(ctx: &Context) -> Vec<Finding> {
    // Yalnızca kullanıcı kapsamında raporla (aynı girdi iki kez sayılmasın).
    let mut out = Vec::new();
    if let Ok(r) = path_analyze::analyze(ctx.provider, Scope::User) {
        for e in r
            .entries
            .iter()
            .filter(|e| e.flags.contains(&Flag::DuplicateOtherScope))
        {
            out.push(f(
                "path.crossScopeDuplicate",
                "path",
                e.raw.trim(),
                Some(Scope::User),
            ));
        }
    }
    out
}

fn path_length(ctx: &Context) -> Vec<Finding> {
    let mut out = Vec::new();
    for v in ctx.persistent().filter(|v| name_key(&v.name) == "PATH") {
        let len = v.value.encode_utf16().count();
        if len > path_analyze::LEGACY_LEN {
            out.push(with(
                with(
                    f("path.length", "path", &v.name, Some(v.scope)),
                    "length",
                    len,
                ),
                "limit",
                path_analyze::LEGACY_LEN,
            ));
        }
    }
    let merged = ctx
        .effective
        .get("PATH")
        .map(|(_, v)| v.encode_utf16().count())
        .unwrap_or(0);
    if merged > 8191 {
        out.push(with(
            with(f("path.length", "path", "PATH", None), "length", merged),
            "limit",
            8191,
        ));
    }
    out
}

// ---------- Değişkenler ----------

fn var_issue(ctx: &Context, issue: Issue, rule: &'static str) -> Vec<Finding> {
    ctx.persistent()
        .filter(|v| v.kind != ValueKind::Other && lint(&v.name, &v.value, v.kind).contains(&issue))
        .map(|v| f(rule, "variables", &v.name, Some(v.scope)))
        .collect()
}

fn var_name(ctx: &Context) -> Vec<Finding> {
    let mut v = var_issue(ctx, Issue::NameWhitespace, "var.name");
    v.extend(var_issue(ctx, Issue::NameLeadingDigit, "var.name"));
    v
}
fn var_whitespace(ctx: &Context) -> Vec<Finding> {
    var_issue(ctx, Issue::ValueWhitespace, "var.whitespace")
}
fn var_unexpanded(ctx: &Context) -> Vec<Finding> {
    var_issue(ctx, Issue::UnexpandedReference, "var.unexpanded")
}
fn var_empty(ctx: &Context) -> Vec<Finding> {
    var_issue(ctx, Issue::EmptyValue, "var.empty")
}

fn var_undefined_reference(ctx: &Context) -> Vec<Finding> {
    let mut out = Vec::new();
    for v in ctx
        .persistent()
        .filter(|v| v.kind == ValueKind::Expand || has_percent_reference(&v.value))
    {
        if view_of(v, ctx.overrides).sensitive {
            continue;
        }
        for r in references(&v.value) {
            if !ctx.effective.contains_key(&name_key(&r)) {
                out.push(with(
                    f(
                        "var.undefinedReference",
                        "variables",
                        &v.name,
                        Some(v.scope),
                    ),
                    "reference",
                    r,
                ));
            }
        }
    }
    out
}

fn var_shadowed(ctx: &Context) -> Vec<Finding> {
    let machine: HashMap<String, &EnvVar> =
        ctx.machine.iter().map(|v| (name_key(&v.name), v)).collect();
    ctx.user
        .iter()
        // TEMP/TMP Windows tarafından kasıtlı olarak kullanıcı başına tanımlanır.
        .filter(|u| !["PATH", "TEMP", "TMP"].contains(&name_key(&u.name).as_str()))
        .filter(|u| {
            machine
                .get(&name_key(&u.name))
                .is_some_and(|m| m.value != u.value)
        })
        .map(|u| f("var.shadowed", "variables", &u.name, Some(Scope::User)))
        .collect()
}

fn home_like(name: &str) -> bool {
    let n = name_key(name);
    (n.ends_with("_HOME") || n.ends_with("_ROOT") || n.ends_with("_DIR"))
        && !n.starts_with("ONEDRIVE")
}

fn expanded(ctx: &Context, v: &EnvVar) -> String {
    if v.kind == ValueKind::Expand {
        expand_references(&v.value, &effective::lookup(&ctx.effective))
    } else {
        v.value.clone()
    }
}

fn var_home_missing(ctx: &Context) -> Vec<Finding> {
    ctx.persistent()
        .filter(|v| home_like(&v.name) && !view_of(v, ctx.overrides).sensitive)
        .filter_map(|v| {
            let x = expanded(ctx, v);
            let x = x.trim().trim_matches('"');
            if x.is_empty() || is_relative_path(x) || x.contains('%') || x.contains(';') {
                return None;
            }
            (!Path::new(x).exists()).then(|| f("var.homeMissing", "tools", &v.name, Some(v.scope)))
        })
        .collect()
}

/// `JAVA_HOME`, `GOROOT` gibi araç kökleri tanımlıysa ama `bin` dizinleri PATH'te değilse.
fn tool_home_not_in_path(ctx: &Context) -> Vec<Finding> {
    const HOMES: &[(&str, &str)] = &[
        ("JAVA_HOME", "bin"),
        ("GOROOT", "bin"),
        ("CARGO_HOME", "bin"),
        ("MAVEN_HOME", "bin"),
        ("GRADLE_HOME", "bin"),
    ];
    let path_keys: HashSet<String> = ctx
        .effective
        .get("PATH")
        .map(|(_, v)| {
            crate::pathmgr::entries::split(v)
                .iter()
                .map(|e| crate::pathmgr::entries::key(e))
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for (name, sub) in HOMES {
        let Some(v) = ctx
            .user
            .iter()
            .find(|v| name_key(&v.name) == *name)
            .or_else(|| ctx.machine.iter().find(|v| name_key(&v.name) == *name))
        else {
            continue;
        };
        let home = expanded(ctx, v);
        let bin = Path::new(home.trim()).join(sub);
        if bin.is_dir()
            && !path_keys.contains(&crate::pathmgr::entries::key(&bin.to_string_lossy()))
        {
            out.push(with(
                f("tools.homeNotInPath", "tools", &v.name, Some(v.scope)),
                "dir",
                bin.display(),
            ));
        }
    }
    out
}

fn temp_dirs(ctx: &Context) -> Vec<Finding> {
    ["TEMP", "TMP"]
        .iter()
        .filter_map(|n| {
            let (name, value) = ctx.effective.get(*n)?;
            (!Path::new(value.trim()).is_dir())
                .then(|| f("system.tempMissing", "system", name, None))
        })
        .collect()
}

fn pathext(ctx: &Context) -> Vec<Finding> {
    let Some((name, value)) = ctx.effective.get("PATHEXT") else {
        return vec![f("system.pathext", "system", "PATHEXT", None)];
    };
    let exts: HashSet<String> = value.split(';').map(|s| s.trim().to_uppercase()).collect();
    let missing: Vec<&str> = [".EXE", ".COM", ".BAT", ".CMD"]
        .into_iter()
        .filter(|e| !exts.contains(*e))
        .collect();
    if missing.is_empty() {
        Vec::new()
    } else {
        vec![with(
            f("system.pathext", "system", name, None),
            "missing",
            missing.join(", "),
        )]
    }
}

// ---------- Güvenlik ----------

fn secrets_in_machine(ctx: &Context) -> Vec<Finding> {
    ctx.machine
        .iter()
        .filter(|v| view_of(v, ctx.overrides).sensitive)
        .map(|v| {
            f(
                "security.machineSecret",
                "security",
                &v.name,
                Some(Scope::Machine),
            )
        })
        .collect()
}

fn secrets_persistent(ctx: &Context) -> Vec<Finding> {
    ctx.user
        .iter()
        .filter(|v| view_of(v, ctx.overrides).sensitive)
        .map(|v| {
            f(
                "security.persistentSecret",
                "security",
                &v.name,
                Some(Scope::User),
            )
        })
        .collect()
}

// ---------- İşlem ve profiller ----------

fn process_stale(ctx: &Context) -> Vec<Finding> {
    let process: HashMap<String, &str> = ctx
        .process
        .iter()
        .map(|v| (name_key(&v.name), v.value.as_str()))
        .collect();
    let mut changed = 0;
    for (k, (_, v)) in &ctx.effective {
        if k == "PATH" {
            continue;
        }
        if ctx.persistent().any(|p| name_key(&p.name) == *k)
            && process.get(k).is_none_or(|pv| pv != v)
        {
            changed += 1;
        }
    }
    if changed > 0 {
        vec![with(
            f("process.stale", "process", "Ivily Envora", None),
            "count",
            changed,
        )]
    } else {
        Vec::new()
    }
}

fn active_profiles(ctx: &Context) -> Vec<(i64, String)> {
    ctx
        .db
        .with(|c| {
            let mut st = c.prepare("SELECT DISTINCT a.profile_id, p.name FROM profile_activations a JOIN profiles p ON p.id = a.profile_id WHERE a.deactivated_at IS NULL")?;
            let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
        .unwrap_or_default()
}

/// Etkin profillerin aynı kapsamda aynı değişkeni farklı değerlerle ayarlaması.
fn profile_conflicts(ctx: &Context) -> Vec<Finding> {
    let mut seen: HashMap<(Scope, String), (String, Option<String>)> = HashMap::new();
    let mut out = Vec::new();
    for (id, pname) in active_profiles(ctx) {
        let Ok(p) = crate::profiles::store::load_full(ctx.db, id) else {
            continue;
        };
        for v in p.vars.iter().filter(|v| v.scope.is_persistent()) {
            let key = (v.scope, name_key(&v.name));
            match seen.get(&key) {
                Some((other, val)) if *val != v.value => {
                    out.push(with(
                        with(
                            f("profile.conflict", "profiles", &v.name, Some(v.scope)),
                            "a",
                            other,
                        ),
                        "b",
                        &pname,
                    ));
                }
                _ => {
                    seen.insert(key, (pname.clone(), v.value.clone()));
                }
            }
        }
    }
    out
}

/// Etkin bir profilin değerleri ortamda artık farklıysa (ör. elle değiştirilmiş).
fn profile_drift(ctx: &Context) -> Vec<Finding> {
    let mut out = Vec::new();
    for (id, pname) in active_profiles(ctx) {
        let Ok(p) = crate::profiles::store::load_full(ctx.db, id) else {
            continue;
        };
        for v in p.vars.iter().filter(|v| {
            v.scope.is_persistent() && v.action == crate::profiles::model::VarAction::Set
        }) {
            let current = if v.scope == Scope::User {
                &ctx.user
            } else {
                &ctx.machine
            };
            let cur = current
                .iter()
                .find(|c| name_key(&c.name) == name_key(&v.name));
            if cur.map(|c| Some(&c.value)) != Some(v.value.as_ref()) {
                out.push(with(
                    f("profile.drift", "profiles", &v.name, Some(v.scope)),
                    "profile",
                    &pname,
                ));
            }
        }
    }
    out
}

/// Microsoft Store uygulama takma adlarının gerçek kurulumları gölgelemesi (ör. python.exe).
fn store_alias_shadowing(ctx: &Context) -> Vec<Finding> {
    const WATCH: &[&str] = &["python", "python3", "py", "winget", "pip"];
    path_analyze::conflicts(ctx.provider)
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            WATCH.contains(&c.name.as_str())
                && c.locations.first().is_some_and(|l| l.app_alias)
                && c.locations.len() > 1
        })
        .map(|c| f("tools.storeAlias", "tools", format!("{}.exe", c.name), None))
        .collect()
}

pub static RULES: &[Rule] = &[
    Rule {
        id: "path.missing",
        category: "path",
        severity: Severity::Warning,
        run: path_missing,
    },
    Rule {
        id: "path.duplicate",
        category: "path",
        severity: Severity::Info,
        run: path_duplicate,
    },
    Rule {
        id: "path.crossScopeDuplicate",
        category: "path",
        severity: Severity::Info,
        run: path_cross_scope,
    },
    Rule {
        id: "path.relative",
        category: "path",
        severity: Severity::Warning,
        run: path_relative,
    },
    Rule {
        id: "path.empty",
        category: "path",
        severity: Severity::Info,
        run: path_empty,
    },
    Rule {
        id: "path.unresolved",
        category: "path",
        severity: Severity::Warning,
        run: path_unresolved,
    },
    Rule {
        id: "path.length",
        category: "path",
        severity: Severity::Warning,
        run: path_length,
    },
    Rule {
        id: "var.name",
        category: "variables",
        severity: Severity::Warning,
        run: var_name,
    },
    Rule {
        id: "var.whitespace",
        category: "variables",
        severity: Severity::Warning,
        run: var_whitespace,
    },
    Rule {
        id: "var.unexpanded",
        category: "variables",
        severity: Severity::Warning,
        run: var_unexpanded,
    },
    Rule {
        id: "var.undefinedReference",
        category: "variables",
        severity: Severity::Warning,
        run: var_undefined_reference,
    },
    Rule {
        id: "var.empty",
        category: "variables",
        severity: Severity::Info,
        run: var_empty,
    },
    Rule {
        id: "var.shadowed",
        category: "variables",
        severity: Severity::Info,
        run: var_shadowed,
    },
    Rule {
        id: "var.homeMissing",
        category: "tools",
        severity: Severity::Warning,
        run: var_home_missing,
    },
    Rule {
        id: "tools.homeNotInPath",
        category: "tools",
        severity: Severity::Info,
        run: tool_home_not_in_path,
    },
    Rule {
        id: "tools.storeAlias",
        category: "tools",
        severity: Severity::Warning,
        run: store_alias_shadowing,
    },
    Rule {
        id: "system.tempMissing",
        category: "system",
        severity: Severity::Error,
        run: temp_dirs,
    },
    Rule {
        id: "system.pathext",
        category: "system",
        severity: Severity::Error,
        run: pathext,
    },
    Rule {
        id: "security.machineSecret",
        category: "security",
        severity: Severity::Warning,
        run: secrets_in_machine,
    },
    Rule {
        id: "security.persistentSecret",
        category: "security",
        severity: Severity::Info,
        run: secrets_persistent,
    },
    Rule {
        id: "process.stale",
        category: "process",
        severity: Severity::Info,
        run: process_stale,
    },
    Rule {
        id: "profile.conflict",
        category: "profiles",
        severity: Severity::Warning,
        run: profile_conflicts,
    },
    Rule {
        id: "profile.drift",
        category: "profiles",
        severity: Severity::Info,
        run: profile_drift,
    },
];
