//! Tauri komut katmanı.
//!
//! Komutlar incedir: girdiyi doğrular, ilgili servisi çağırır ve sonucu döndürür.
//! İş mantığı servis modüllerinde bulunur; böylece komutsuz birim testleri yazılabilir.

pub mod app;
pub mod settings;
