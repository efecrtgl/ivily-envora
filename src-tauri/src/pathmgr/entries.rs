//! PATH girdisi işlemleri: ayırma, birleştirme, karşılaştırma anahtarı ve değişiklik uygulama.

use serde::{Deserialize, Serialize};

/// PATH değerini girdilere ayırır. Boş girdiler korunur (analiz bunları raporlar).
pub fn split(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    value.split(';').map(String::from).collect()
}

pub fn join(entries: &[String]) -> String {
    entries.join(";")
}

/// Karşılaştırma anahtarı: büyük/küçük harf, `/` ↔ `\`, sondaki `\` ve çevreleyen tırnak/boşluk yok sayılır.
/// `%VAR%` başvuruları genişletilmez (aynı dizini gösteren farklı yazımlar ayrı sayılır).
pub fn key(entry: &str) -> String {
    let mut s = entry
        .trim()
        .trim_matches('"')
        .replace('/', "\\")
        .to_lowercase();
    while s.len() > 3 && s.ends_with('\\') {
        s.pop();
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathAction {
    /// Başa ekle (zaten varsa başa taşı).
    Prepend,
    /// Sona ekle (zaten varsa yerinde bırak).
    Append,
    /// Kaldır (eşleşen tüm girdiler).
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathMod {
    pub entry: String,
    pub action: PathAction,
}

/// Değişiklikleri sırayla uygular. Boş girdiler (`;;`) korunur; eklenen girdiler boş olamaz.
pub fn apply_mods(current: &str, mods: &[PathMod]) -> String {
    let mut entries = split(current);
    let mut prepend_at = 0usize;
    for m in mods {
        let e = m.entry.trim();
        if e.is_empty() {
            continue;
        }
        let k = key(e);
        match m.action {
            PathAction::Remove => entries.retain(|x| key(x) != k),
            PathAction::Append => {
                if !entries.iter().any(|x| key(x) == k) {
                    entries.push(e.to_string());
                }
            }
            PathAction::Prepend => {
                entries.retain(|x| key(x) != k);
                let at = prepend_at.min(entries.len());
                entries.insert(at, e.to_string());
                // Birden çok "başa ekle" verildiğindeki sırayı koru.
                prepend_at = at + 1;
            }
        }
    }
    join(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(entry: &str, action: PathAction) -> PathMod {
        PathMod {
            entry: entry.into(),
            action,
        }
    }

    #[test]
    fn split_and_key() {
        assert_eq!(split("a;;b"), vec!["a", "", "b"]);
        assert!(split("").is_empty());
        assert_eq!(key("C:/Tools/Bin\\"), "c:\\tools\\bin");
        assert_eq!(key("\"C:\\X\""), "c:\\x");
        assert_eq!(key("C:\\"), "c:\\", "kök dizinin ters eğik çizgisi korunur");
    }

    #[test]
    fn apply_prepend_append_remove() {
        let cur = "C:\\Windows;C:\\Tools;C:\\Old";
        let out = apply_mods(
            cur,
            &[
                m("C:\\jdk\\bin", PathAction::Prepend),
                m("c:\\tools\\", PathAction::Prepend),
                m("C:\\Old", PathAction::Remove),
                m("C:\\Windows", PathAction::Append),
                m("D:\\new", PathAction::Append),
            ],
        );
        assert_eq!(out, "C:\\jdk\\bin;c:\\tools\\;C:\\Windows;D:\\new");
    }

    #[test]
    fn idempotent() {
        let mods = [
            m("C:\\a", PathAction::Prepend),
            m("C:\\b", PathAction::Append),
        ];
        let once = apply_mods("C:\\x", &mods);
        assert_eq!(apply_mods(&once, &mods), once);
    }
}
