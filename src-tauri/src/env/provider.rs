//! Ortam sağlayıcı soyutlaması.
//!
//! Gerçek Windows ortamı `WindowsProvider` ile, testler ise `MemoryProvider` ile çalışır.
//! Otomatik testler gerçek kullanıcı/sistem ortamını **asla** değiştirmez.

use super::model::{name_key, EnvVar, Scope, ValueKind};
use crate::error::{AppError, AppResult};
use parking_lot::Mutex;
use std::collections::BTreeMap;

pub trait EnvProvider: Send + Sync {
    fn list(&self, scope: Scope) -> AppResult<Vec<EnvVar>>;

    fn get(&self, scope: Scope, name: &str) -> AppResult<Option<EnvVar>> {
        let key = name_key(name);
        Ok(self
            .list(scope)?
            .into_iter()
            .find(|v| name_key(&v.name) == key))
    }

    /// Kalıcı kapsamda bir değişkeni yazar. İşlem kapsamı desteklenmez.
    fn set(&self, scope: Scope, name: &str, value: &str, kind: ValueKind) -> AppResult<()>;

    /// Kalıcı kapsamdan bir değişkeni siler. Değişken yoksa hata vermez.
    fn delete(&self, scope: Scope, name: &str) -> AppResult<()>;

    /// Kalıcı değişikliklerden sonra açık pencerelere `WM_SETTINGCHANGE` yayınlar.
    fn broadcast_change(&self) {}

    /// Bu süreç sistem kapsamına doğrudan yazabilir mi (yönetici olarak mı çalışıyor)?
    fn can_write_machine(&self) -> bool {
        false
    }

    /// Sağlayıcının adı (tanılama için).
    fn name(&self) -> &'static str;
}

/// Testler ve önizleme için bellek içi sağlayıcı.
#[derive(Default)]
pub struct MemoryProvider {
    data: Mutex<BTreeMap<(Scope, String), EnvVar>>,
    machine_writable: bool,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_machine_write(mut self, allowed: bool) -> Self {
        self.machine_writable = allowed;
        self
    }

    pub fn seed(&self, scope: Scope, name: &str, value: &str, kind: ValueKind) {
        self.data.lock().insert(
            (scope, name_key(name)),
            EnvVar::new(scope, name, value, kind),
        );
    }
}

impl EnvProvider for MemoryProvider {
    fn list(&self, scope: Scope) -> AppResult<Vec<EnvVar>> {
        Ok(self
            .data
            .lock()
            .iter()
            .filter(|((s, _), _)| *s == scope)
            .map(|(_, v)| v.clone())
            .collect())
    }

    fn set(&self, scope: Scope, name: &str, value: &str, kind: ValueKind) -> AppResult<()> {
        if scope == Scope::Process {
            return Err(AppError::Unsupported(
                "işlem kapsamı kalıcı olarak yazılamaz".into(),
            ));
        }
        if scope == Scope::Machine && !self.machine_writable {
            return Err(AppError::PermissionDenied(
                "sistem kapsamı yönetici yetkisi gerektirir".into(),
            ));
        }
        let mut d = self.data.lock();
        // Var olan değişkenin yazımını (büyük/küçük harf) koru.
        let display = d
            .get(&(scope, name_key(name)))
            .map(|v| v.name.clone())
            .unwrap_or_else(|| name.to_string());
        d.insert(
            (scope, name_key(name)),
            EnvVar::new(scope, &display, value, kind),
        );
        Ok(())
    }

    fn delete(&self, scope: Scope, name: &str) -> AppResult<()> {
        if scope == Scope::Process {
            return Err(AppError::Unsupported(
                "işlem kapsamı kalıcı olarak yazılamaz".into(),
            ));
        }
        if scope == Scope::Machine && !self.machine_writable {
            return Err(AppError::PermissionDenied(
                "sistem kapsamı yönetici yetkisi gerektirir".into(),
            ));
        }
        self.data.lock().remove(&(scope, name_key(name)));
        Ok(())
    }

    fn can_write_machine(&self) -> bool {
        self.machine_writable
    }

    fn name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_provider_is_case_insensitive_and_scoped() {
        let p = MemoryProvider::new();
        p.seed(Scope::User, "Path", "C:\\a", ValueKind::Expand);
        p.seed(Scope::Machine, "PATH", "C:\\b", ValueKind::Expand);
        assert_eq!(p.get(Scope::User, "PATH").unwrap().unwrap().value, "C:\\a");
        p.set(Scope::User, "path", "C:\\c", ValueKind::Expand)
            .unwrap();
        let v = p.get(Scope::User, "Path").unwrap().unwrap();
        assert_eq!(v.value, "C:\\c");
        assert_eq!(v.name, "Path", "mevcut adın yazımı korunmalı");
        assert_eq!(
            p.get(Scope::Machine, "Path").unwrap().unwrap().value,
            "C:\\b",
            "diğer kapsam etkilenmemeli"
        );
    }

    #[test]
    fn memory_provider_enforces_scope_rules() {
        let p = MemoryProvider::new();
        assert!(matches!(
            p.set(Scope::Process, "A", "1", ValueKind::String),
            Err(AppError::Unsupported(_))
        ));
        assert!(matches!(
            p.set(Scope::Machine, "A", "1", ValueKind::String),
            Err(AppError::PermissionDenied(_))
        ));
        let p = MemoryProvider::new().with_machine_write(true);
        p.set(Scope::Machine, "A", "1", ValueKind::String).unwrap();
        p.delete(Scope::Machine, "a").unwrap();
        assert!(p.get(Scope::Machine, "A").unwrap().is_none());
    }
}
