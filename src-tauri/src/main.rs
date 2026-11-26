// Sürüm derlemelerinde Windows'ta ek konsol penceresi açılmasını engeller.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Sistem kapsamı için UAC ile başlatılan yardımcı kip: arayüz açmadan planı uygular ve çıkar.
    if let Some(code) = envora_lib::env::elevate::maybe_run_helper_from_args() {
        std::process::exit(code);
    }
    envora_lib::run()
}
