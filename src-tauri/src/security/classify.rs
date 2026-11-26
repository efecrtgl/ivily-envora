//! Hassas değişken sınıflandırması.
//!
//! Sınıflandırma; ad parçalarına (ör. `*_TOKEN`, `*SECRET*`) ve bilinen gizli değer biçimlerine
//! (ör. GitHub, AWS, Stripe anahtar önekleri, PEM başlıkları, parola içeren URL'ler) dayanır.
//! Her değişkeni gizli saymaz; kullanıcı her ad için sınıflandırmayı geçersiz kılabilir
//! (bkz. `overrides`). Kurallar bir tablo olarak tutulur; yeni kural eklemek tek satırdır.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    /// Ad bir gizli bilgi anahtar kelimesi içeriyor.
    Name,
    /// Değer bilinen bir gizli anahtar biçimine benziyor.
    ValueFormat,
    /// Değer, parola içeren bir bağlantı adresi/dizesi.
    Credentials,
    /// Kullanıcı tarafından hassas olarak işaretlendi.
    UserOverride,
}

/// Ad parçası olarak tam eşleşen anahtar kelimeler (`_`, `-`, `.` ile ayrılmış parçalar).
const NAME_TOKENS: &[&str] = &[
    "SECRET",
    "SECRETS",
    "PASSWORD",
    "PASSWD",
    "PASS",
    "PASSPHRASE",
    "TOKEN",
    "TOKENS",
    "APIKEY",
    "CREDENTIAL",
    "CREDENTIALS",
    "CREDS",
    "COOKIE",
    "SIGNING",
    "AUTH",
    "BEARER",
    "DSN",
    "CONNSTR",
    "CONNECTIONSTRING",
    "SAS",
    "WEBHOOK",
    "PAT",
    "OTP",
    "SALT",
    "HMAC",
];

/// Ardışık parça çiftleri.
const NAME_PAIRS: &[(&str, &str)] = &[
    ("API", "KEY"),
    ("ACCESS", "KEY"),
    ("SECRET", "KEY"),
    ("PRIVATE", "KEY"),
    ("SESSION", "KEY"),
    ("SESSION", "SECRET"),
    ("CLIENT", "SECRET"),
    ("ACCOUNT", "KEY"),
    ("MASTER", "KEY"),
    ("ENCRYPTION", "KEY"),
    ("CONNECTION", "STRING"),
    ("DATABASE", "URL"),
    ("DB", "URL"),
    ("REDIS", "URL"),
    ("MONGO", "URL"),
    ("MONGODB", "URI"),
];

/// Hassas görünse de olmayan iyi bilinen adlar.
const NAME_ALLOW: &[&str] = &[
    "PWD",
    "OLDPWD",
    "PATH",
    "PATHEXT",
    "PASSWORD_STORE_DIR",
    "SESSIONNAME",
    "TOKENIZERS_PARALLELISM",
    "GIT_ASKPASS",
    "SSH_ASKPASS",
    "SSH_AUTH_SOCK",
    "GPG_TTY",
    "AUTH_TYPE",
];

/// Değer önekleri: (önek, öneki izlemesi gereken en az karakter sayısı)
const VALUE_PREFIXES: &[(&str, usize)] = &[
    ("ghp_", 20),
    ("gho_", 20),
    ("ghu_", 20),
    ("ghs_", 20),
    ("ghr_", 20),
    ("github_pat_", 20),
    ("glpat-", 16),
    ("sk-", 20),
    ("sk_live_", 16),
    ("sk_test_", 16),
    ("rk_live_", 16),
    ("pk_live_", 16),
    ("xoxb-", 10),
    ("xoxp-", 10),
    ("xapp-", 10),
    ("AKIA", 16),
    ("ASIA", 16),
    ("AIza", 30),
    ("npm_", 30),
    ("pypi-", 30),
    ("hf_", 30),
    ("SG.", 30),
    ("dop_v1_", 30),
    ("shpat_", 20),
];

fn split_tokens(name: &str) -> Vec<String> {
    name.to_uppercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

pub fn name_reason(name: &str) -> Option<Reason> {
    let upper = name.to_uppercase();
    if NAME_ALLOW.contains(&upper.as_str()) {
        return None;
    }
    let tokens = split_tokens(name);
    if tokens.iter().any(|t| NAME_TOKENS.contains(&t.as_str())) {
        return Some(Reason::Name);
    }
    if tokens
        .windows(2)
        .any(|w| NAME_PAIRS.iter().any(|(a, b)| w[0] == *a && w[1] == *b))
    {
        return Some(Reason::Name);
    }
    // Tek parça birleşik adlar: GITHUBTOKEN, MYAPISECRET …
    if tokens.len() == 1 {
        let t = &tokens[0];
        if ["TOKEN", "SECRET", "PASSWORD", "APIKEY"]
            .iter()
            .any(|k| t.ends_with(k) && t.len() > k.len())
        {
            return Some(Reason::Name);
        }
    }
    None
}

fn looks_like_jwt(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && v.starts_with("eyJ")
        && parts.iter().all(|p| {
            p.len() >= 4
                && p.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

fn url_has_password(v: &str) -> bool {
    // şema://kullanıcı:parola@sunucu
    let Some(idx) = v.find("://") else {
        return false;
    };
    let rest = &v[idx + 3..];
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    match authority.rfind('@') {
        Some(at) => authority[..at]
            .split_once(':')
            .is_some_and(|(_, p)| !p.is_empty()),
        None => false,
    }
}

fn conn_string_has_secret(v: &str) -> bool {
    let lower = v.to_lowercase();
    [
        "password=",
        "pwd=",
        "accountkey=",
        "sharedaccesskey=",
        "secret=",
    ]
    .iter()
    .any(|k| {
        lower
            .split(';')
            .any(|part| part.trim_start().starts_with(k) && part.trim().len() > k.len())
    })
}

pub fn value_reason(value: &str) -> Option<Reason> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    if v.contains("-----BEGIN") && v.contains("PRIVATE KEY") {
        return Some(Reason::ValueFormat);
    }
    for (prefix, min) in VALUE_PREFIXES {
        if let Some(rest) = v.strip_prefix(prefix) {
            if rest.len() >= *min
                && !rest.contains(char::is_whitespace)
                && !rest.contains(['\\', ';'])
            {
                return Some(Reason::ValueFormat);
            }
        }
    }
    if looks_like_jwt(v) {
        return Some(Reason::ValueFormat);
    }
    if url_has_password(v) || conn_string_has_secret(v) {
        return Some(Reason::Credentials);
    }
    None
}

/// Algılama sonucu (kullanıcı geçersiz kılması uygulanmadan önce).
pub fn detect(name: &str, value: &str) -> Option<Reason> {
    name_reason(name).or_else(|| value_reason(value))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sensitivity {
    pub sensitive: bool,
    pub reason: Option<Reason>,
    /// Kullanıcı geçersiz kılması uygulandı mı?
    pub overridden: bool,
}

/// Algılamayı ve kullanıcı geçersiz kılmasını birleştirir.
pub fn resolve(name: &str, value: &str, user_override: Option<bool>) -> Sensitivity {
    match user_override {
        Some(true) => Sensitivity {
            sensitive: true,
            reason: Some(Reason::UserOverride),
            overridden: true,
        },
        Some(false) => Sensitivity {
            sensitive: false,
            reason: None,
            overridden: true,
        },
        None => {
            let reason = detect(name, value);
            Sensitivity {
                sensitive: reason.is_some(),
                reason,
                overridden: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_names() {
        for n in [
            "GITHUB_TOKEN",
            "OPENAI_API_KEY",
            "AWS_SECRET_ACCESS_KEY",
            "DB_PASSWORD",
            "CLIENT_SECRET",
            "DATABASE_URL",
            "NPM_AUTH_TOKEN",
            "SENTRY_DSN",
            "githubtoken",
            "Stripe-Secret",
            "AZURE_CLIENT_SECRET",
            "MY_PRIVATE_KEY",
            "SIGNING_KEY_PASSPHRASE",
            "TAURI_SIGNING_PRIVATE_KEY",
        ] {
            assert_eq!(name_reason(n), Some(Reason::Name), "{n} hassas olmalı");
        }
    }

    #[test]
    fn non_sensitive_names() {
        for n in [
            "PATH",
            "PWD",
            "OLDPWD",
            "JAVA_HOME",
            "TOKENIZERS_PARALLELISM",
            "SESSIONNAME",
            "USERNAME",
            "PROCESSOR_IDENTIFIER",
            "KEYBOARD_LAYOUT",
            "PASSWORD_STORE_DIR",
            "PATHEXT",
            "SSH_AUTH_SOCK",
            "COMPASS",
            "BYPASS_PROXY",
            "CLAUDE_CODE_CHILD_SESSION",
            "SESSION_ID",
            "PRIVATE_REGISTRY",
            "MONKEY",
        ] {
            assert_eq!(name_reason(n), None, "{n} hassas olmamalı");
        }
    }

    #[test]
    fn sensitive_values() {
        let fake = |p: &str, n: usize| format!("{p}{}", "A1b2".repeat(n));
        assert!(value_reason(&fake("ghp_", 9)).is_some());
        assert!(value_reason(&fake(concat!("github", "_pat_"), 9)).is_some());
        assert!(value_reason(&fake("sk-", 9)).is_some());
        assert!(value_reason(&fake("AKIA", 4)).is_some());
        assert!(value_reason(concat!("-----BEGIN OPENSSH ", "PRIVATE KEY-----\nabc")).is_some());
        assert!(value_reason("eyJhbGciOi.eyJzdWIiOi.c2lnbmF0dXJl").is_some());
        assert_eq!(
            value_reason("postgres://app:hunter2@db:5432/x"),
            Some(Reason::Credentials)
        );
        assert_eq!(
            value_reason("Server=x;Database=y;Password=hunter2;"),
            Some(Reason::Credentials)
        );
    }

    #[test]
    fn non_sensitive_values() {
        for v in [
            "C:\\Windows\\system32",
            "sk-",
            "https://example.com/path",
            "https://user@host/x",
            "Password=;",
            "AMD64",
            "%USERPROFILE%\\.cargo\\bin",
            "ghp_short",
            "eyJ.not.jwt",
            "skip-this-value-please",
        ] {
            assert_eq!(value_reason(v), None, "{v} hassas olmamalı");
        }
    }

    #[test]
    fn override_wins() {
        assert!(!resolve("GITHUB_TOKEN", "x", Some(false)).sensitive);
        let s = resolve("FOO", "bar", Some(true));
        assert!(s.sensitive && s.overridden && s.reason == Some(Reason::UserOverride));
        assert!(resolve("GITHUB_TOKEN", "x", None).sensitive);
    }
}
