//! Tauri komut katmanı.
//!
//! Komutlar incedir: girdiyi doğrular, ilgili servisi çağırır ve sonucu döndürür.
//! İş mantığı servis modüllerinde bulunur; böylece komutsuz birim testleri yazılabilir.

pub mod app;
pub mod env;
pub mod profiles;
pub mod projects;
pub mod settings;
