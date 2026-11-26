use super::*;
use crate::env::provider::MemoryProvider;
use crate::env::ValueKind;

fn ids(r: &Report) -> Vec<&'static str> {
    r.findings.iter().map(|f| f.rule).collect()
}

fn base() -> MemoryProvider {
    let p = MemoryProvider::new();
    let tmp = std::env::temp_dir().to_string_lossy().to_string();
    p.seed(Scope::Process, "TEMP", &tmp, ValueKind::String);
    p.seed(Scope::Process, "TMP", &tmp, ValueKind::String);
    p.seed(
        Scope::Process,
        "PATHEXT",
        ".COM;.EXE;.BAT;.CMD",
        ValueKind::String,
    );
    p
}

#[test]
fn clean_environment_has_no_findings() {
    let db = Db::open_in_memory().unwrap();
    let p = base();
    let r = run(&db, &p, &Overrides::new(), &Config::default()).unwrap();
    assert!(r.findings.is_empty(), "{:?}", ids(&r));
    assert_eq!(r.rules_run, rules().len());
}

#[test]
fn detects_problems_without_leaking_values() {
    let db = Db::open_in_memory().unwrap();
    let p = base();
    p.seed(
        Scope::User,
        "Path",
        "Z:\\envora\\missing;relative;;Z:\\envora\\missing",
        ValueKind::Expand,
    );
    p.seed(Scope::User, "PADDED", " value ", ValueKind::String);
    p.seed(Scope::User, "NOEXP", "%USERPROFILE%\\x", ValueKind::String);
    p.seed(Scope::User, "REF", "%DOES_NOT_EXIST%\\x", ValueKind::Expand);
    p.seed(
        Scope::User,
        "JAVA_HOME",
        "Z:\\envora\\jdk",
        ValueKind::String,
    );
    p.seed(Scope::User, "EDITOR", "code", ValueKind::String);
    p.seed(Scope::Machine, "EDITOR", "notepad", ValueKind::String);
    p.seed(
        Scope::Machine,
        "DB_PASSWORD",
        "super-secret-pw",
        ValueKind::String,
    );
    let r = run(&db, &p, &Overrides::new(), &Config::default()).unwrap();
    let got = ids(&r);
    for want in [
        "path.missing",
        "path.relative",
        "path.empty",
        "path.duplicate",
        "var.whitespace",
        "var.unexpanded",
        "var.undefinedReference",
        "var.homeMissing",
        "var.shadowed",
        "security.machineSecret",
        "process.stale",
    ] {
        assert!(got.contains(&want), "{want} bekleniyordu: {got:?}");
    }
    assert!(!serde_json::to_string(&r)
        .unwrap()
        .contains("super-secret-pw"));
    assert!(
        r.findings
            .windows(2)
            .all(|w| w[0].severity >= w[1].severity),
        "önem derecesine göre sıralı"
    );
}

#[test]
fn rules_can_be_disabled_and_reweighted() {
    let db = Db::open_in_memory().unwrap();
    let p = base();
    p.seed(Scope::Machine, "API_TOKEN", "x", ValueKind::String);
    let mut cfg = Config::default();
    cfg.severity
        .insert("security.machineSecret".into(), "error".into());
    let r = run(&db, &p, &Overrides::new(), &cfg).unwrap();
    assert_eq!(
        r.findings
            .iter()
            .find(|f| f.rule == "security.machineSecret")
            .unwrap()
            .severity,
        Severity::Error
    );
    cfg.disabled.push("security.machineSecret".into());
    let r = run(&db, &p, &Overrides::new(), &cfg).unwrap();
    assert!(!ids(&r).contains(&"security.machineSecret"));
    assert_eq!(r.rules_run, rules().len() - 1);
}

#[test]
fn system_basics() {
    let db = Db::open_in_memory().unwrap();
    let p = MemoryProvider::new();
    p.seed(
        Scope::Process,
        "TEMP",
        "Z:\\envora\\no-temp",
        ValueKind::String,
    );
    p.seed(Scope::Process, "PATHEXT", ".COM;.BAT", ValueKind::String);
    let r = run(&db, &p, &Overrides::new(), &Config::default()).unwrap();
    assert!(ids(&r).contains(&"system.tempMissing"));
    let pe = r
        .findings
        .iter()
        .find(|f| f.rule == "system.pathext")
        .unwrap();
    assert_eq!(pe.params["missing"], ".EXE, .CMD");
}

#[test]
fn rule_ids_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for r in rules() {
        assert!(seen.insert(r.id), "yinelenen kural kimliği {}", r.id);
    }
}
