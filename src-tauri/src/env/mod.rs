//! Ortam değişkenleri: model, sağlayıcılar, doğrulama, sınıflandırma ve inceleme.

pub mod category;
pub mod inspect;
pub mod model;
pub mod provider;
pub mod validate;
pub mod windows;

pub use model::{EnvVar, Scope, ValueKind};
pub use provider::{EnvProvider, MemoryProvider};
