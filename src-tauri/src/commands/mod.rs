//! Tauri komut katmanı.
//!
//! Komutlar incedir: girdiyi doğrular, ilgili servisi çağırır ve sonucu döndürür.
//! İş mantığı servis modüllerinde bulunur; böylece komutsuz birim testleri yazılabilir.

pub mod activity;
pub mod analysis;
pub mod app;
pub mod compare;
pub mod discovery;
pub mod env;
pub mod io;
pub mod path;
pub mod profiles;
pub mod projects;
pub mod settings;
pub mod snapshots;
