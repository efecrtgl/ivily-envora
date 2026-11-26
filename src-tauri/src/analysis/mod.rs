//! Ortam analizi: modüler, yapılandırılabilir kural motoru.
//!
//! Her kural `RULES` tablosunda bir kimlik, kategori ve varsayılan önem derecesiyle tanımlanır
//! ve tek bir işlevdir. Yeni kural eklemek için `rules.rs` içine bir işlev ve tabloya bir satır
//! eklemek yeterlidir. Kurallar yalnızca okur; hiçbir şeyi düzeltmez.
//!
//! Analiz yalnızca istendiğinde çalışır (sürekli arka plan taraması yoktur).

mod rules;

use crate::db::Db;
use crate::env::effective::{self, EnvMap};
use crate::env::inspect::Overrides;
use crate::env::provider::EnvProvider;
use crate::env::{EnvVar, Scope};
use crate::error::AppResult;
pub use crate::profiles::validate::Severity;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub rule: &'static str,
    pub category: &'static str,
    pub severity: Severity,
    /// Bulgunun ilgili olduğu ad (değişken adı, PATH girdisi …). Asla değişken değeri değildir.
    pub target: String,
    pub scope: Option<Scope>,
    /// Çeviri için ek parametreler (sayılar, kısa adlar).
    pub params: BTreeMap<&'static str, String>,
}

pub struct Context<'a> {
    pub user: Vec<EnvVar>,
    pub machine: Vec<EnvVar>,
    pub process: Vec<EnvVar>,
    pub effective: EnvMap,
    pub overrides: &'a Overrides,
    pub db: &'a Db,
    pub provider: &'a dyn EnvProvider,
}

impl Context<'_> {
    pub fn persistent(&self) -> impl Iterator<Item = &EnvVar> {
        self.user.iter().chain(self.machine.iter())
    }
}

type RuleFn = fn(&Context) -> Vec<Finding>;

pub struct Rule {
    pub id: &'static str,
    pub category: &'static str,
    pub severity: Severity,
    pub run: RuleFn,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInfo {
    pub id: &'static str,
    pub category: &'static str,
    pub default_severity: Severity,
    pub enabled: bool,
    pub severity: Severity,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub disabled: Vec<String>,
    pub severity: BTreeMap<String, String>,
}

fn parse_severity(s: &str) -> Option<Severity> {
    match s {
        "info" => Some(Severity::Info),
        "warning" => Some(Severity::Warning),
        "error" => Some(Severity::Error),
        _ => None,
    }
}

impl Config {
    fn enabled(&self, id: &str) -> bool {
        !self.disabled.iter().any(|d| d == id)
    }
    fn severity(&self, r: &Rule) -> Severity {
        self.severity
            .get(r.id)
            .and_then(|s| parse_severity(s))
            .unwrap_or(r.severity)
    }
}

pub fn rules() -> &'static [Rule] {
    rules::RULES
}

pub fn rule_infos(cfg: &Config) -> Vec<RuleInfo> {
    rules()
        .iter()
        .map(|r| RuleInfo {
            id: r.id,
            category: r.category,
            default_severity: r.severity,
            enabled: cfg.enabled(r.id),
            severity: cfg.severity(r),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub findings: Vec<Finding>,
    pub counts: BTreeMap<String, usize>,
    pub rules_run: usize,
    pub generated_at: String,
    pub duration_ms: u128,
}

pub fn run(db: &Db, p: &dyn EnvProvider, overrides: &Overrides, cfg: &Config) -> AppResult<Report> {
    let started = std::time::Instant::now();
    let ctx = Context {
        user: p.list(Scope::User)?,
        machine: p.list(Scope::Machine)?,
        process: p.list(Scope::Process)?,
        effective: effective::fresh_environment(p)?,
        overrides,
        db,
        provider: p,
    };
    let mut findings = Vec::new();
    let mut rules_run = 0;
    for r in rules().iter().filter(|r| cfg.enabled(r.id)) {
        rules_run += 1;
        let sev = cfg.severity(r);
        for mut f in (r.run)(&ctx) {
            f.severity = sev;
            findings.push(f);
        }
    }
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then(a.rule.cmp(b.rule))
            .then(a.target.cmp(&b.target))
    });
    let mut counts = BTreeMap::new();
    for f in &findings {
        let k = serde_json::to_value(f.severity)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        *counts.entry(k).or_insert(0) += 1;
    }
    Ok(Report {
        findings,
        counts,
        rules_run,
        generated_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests;
