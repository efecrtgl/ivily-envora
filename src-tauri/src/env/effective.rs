//! Yeni başlatılan bir işlemin göreceği ortamın yaklaşık hesabı.
//!
//! Explorer gibi bir üst süreç yeni bir program başlattığında ortam; oturum değişkenleri
//! (`USERPROFILE`, `APPDATA`, `SystemRoot` …), sistem kapsamı ve kullanıcı kapsamının
//! birleşimidir. `REG_EXPAND_SZ` değerler genişletilir ve PATH = sistem PATH + kullanıcı PATH olur.
//!
//! Bu modül bunu şöyle yaklaşık hesaplar: temel olarak Envora'nın kendi işlem ortamı
//! (oturum değişkenlerini içerir) alınır, ardından **güncel** kayıt defteri değerleri
//! (önce sistem, sonra kullanıcı) üzerine yazılır. Envora başlatıldıktan sonra yapılan kalıcı
//! değişiklikler böylece yansır. Windows'un oturum açılışında yaptığı birebir işlem değildir.

use super::inspect::expand_references;
use super::model::{name_key, Scope, ValueKind};
use super::provider::EnvProvider;
use crate::error::AppResult;
use std::collections::BTreeMap;

/// Büyük harfli anahtar → (görünen ad, değer)
pub type EnvMap = BTreeMap<String, (String, String)>;

pub fn lookup(map: &EnvMap) -> impl Fn(&str) -> Option<String> + '_ {
    move |n: &str| map.get(&name_key(n)).map(|(_, v)| v.clone())
}

pub fn set(map: &mut EnvMap, name: &str, value: String) {
    let key = name_key(name);
    let display = map
        .get(&key)
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| name.to_string());
    map.insert(key, (display, value));
}

pub fn fresh_environment(p: &dyn EnvProvider) -> AppResult<EnvMap> {
    let mut map = EnvMap::new();
    for v in p.list(Scope::Process)? {
        map.insert(name_key(&v.name), (v.name, v.value));
    }
    let machine = p.list(Scope::Machine)?;
    let user = p.list(Scope::User)?;
    let mut machine_path = String::new();
    let mut user_path = String::new();
    for scope_vars in [&machine, &user] {
        for v in scope_vars.iter() {
            if v.kind == ValueKind::Other {
                continue;
            }
            let value = if v.kind == ValueKind::Expand {
                expand_references(&v.value, &lookup(&map))
            } else {
                v.value.clone()
            };
            if name_key(&v.name) == "PATH" {
                if v.scope == Scope::Machine {
                    machine_path = value;
                } else {
                    user_path = value;
                }
                continue;
            }
            set(&mut map, &v.name, value);
        }
    }
    let path = match (machine_path.is_empty(), user_path.is_empty()) {
        (false, false) => format!(
            "{}{}{user_path}",
            machine_path,
            if machine_path.ends_with(';') { "" } else { ";" }
        ),
        (false, true) => machine_path,
        (true, false) => user_path,
        (true, true) => map.get("PATH").map(|(_, v)| v.clone()).unwrap_or_default(),
    };
    set(&mut map, "Path", path);
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;

    #[test]
    fn merges_scopes_like_a_new_process() {
        let p = MemoryProvider::new();
        p.seed(
            Scope::Process,
            "USERPROFILE",
            "C:\\Users\\u",
            ValueKind::String,
        );
        p.seed(Scope::Process, "Path", "stale", ValueKind::String);
        p.seed(Scope::Process, "EDITOR", "old", ValueKind::String);
        p.seed(Scope::Machine, "Path", "C:\\Windows", ValueKind::Expand);
        p.seed(Scope::Machine, "JAVA_HOME", "C:\\jdk17", ValueKind::String);
        p.seed(Scope::User, "Path", "%USERPROFILE%\\bin", ValueKind::Expand);
        p.seed(Scope::User, "JAVA_HOME", "C:\\jdk21", ValueKind::String);
        p.seed(Scope::User, "EDITOR", "code", ValueKind::String);
        let m = fresh_environment(&p).unwrap();
        let get = |k: &str| m.get(k).map(|(_, v)| v.as_str());
        assert_eq!(get("PATH"), Some("C:\\Windows;C:\\Users\\u\\bin"));
        assert_eq!(
            get("JAVA_HOME"),
            Some("C:\\jdk21"),
            "kullanıcı değeri sistem değerini gölgeler"
        );
        assert_eq!(
            get("EDITOR"),
            Some("code"),
            "güncel kayıt değeri işlemdeki eski değerin yerine geçer"
        );
        assert_eq!(
            m.get("PATH").unwrap().0,
            "Path",
            "işlemdeki adın yazımı korunur"
        );
    }
}
