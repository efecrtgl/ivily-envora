//! Gerçek Windows ortam sağlayıcısı.
//!
//! - Kullanıcı kapsamı: `HKCU\Environment`
//! - Sistem kapsamı: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`
//! - İşlem kapsamı: bu sürecin ortamı (yalnızca okunur)
//!
//! Değer türü (`REG_SZ` / `REG_EXPAND_SZ`) okunurken ve yazılırken korunur.

use super::model::{EnvVar, Scope, ValueKind};
use super::provider::EnvProvider;
use crate::error::{AppError, AppResult};

const USER_KEY: &str = "Environment";
const MACHINE_KEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

pub struct WindowsProvider {
    elevated: bool,
}

impl WindowsProvider {
    pub fn new() -> Self {
        WindowsProvider {
            elevated: imp::is_elevated(),
        }
    }
}

impl Default for WindowsProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn process_vars() -> Vec<EnvVar> {
    std::env::vars_os()
        .filter_map(|(k, v)| {
            let name = k.to_string_lossy().into_owned();
            // Windows, sürücü başına çalışma dizinlerini "=C:" gibi gizli değişkenlerde tutar; listelenmez.
            if name.is_empty() || name.starts_with('=') {
                return None;
            }
            Some(EnvVar::new(
                Scope::Process,
                &name,
                &v.to_string_lossy(),
                ValueKind::String,
            ))
        })
        .collect()
}

impl EnvProvider for WindowsProvider {
    fn list(&self, scope: Scope) -> AppResult<Vec<EnvVar>> {
        match scope {
            Scope::Process => Ok(process_vars()),
            Scope::User => imp::read_key(scope, true, USER_KEY),
            Scope::Machine => imp::read_key(scope, false, MACHINE_KEY),
        }
    }

    fn set(&self, scope: Scope, name: &str, value: &str, kind: ValueKind) -> AppResult<()> {
        match scope {
            Scope::Process => Err(AppError::Unsupported(
                "işlem kapsamı kalıcı olarak yazılamaz".into(),
            )),
            Scope::User => imp::write_value(true, USER_KEY, name, value, kind),
            Scope::Machine => imp::write_value(false, MACHINE_KEY, name, value, kind),
        }
    }

    fn delete(&self, scope: Scope, name: &str) -> AppResult<()> {
        match scope {
            Scope::Process => Err(AppError::Unsupported(
                "işlem kapsamı kalıcı olarak yazılamaz".into(),
            )),
            Scope::User => imp::delete_value(true, USER_KEY, name),
            Scope::Machine => imp::delete_value(false, MACHINE_KEY, name),
        }
    }

    fn broadcast_change(&self) {
        imp::broadcast_environment_change();
    }

    fn can_write_machine(&self) -> bool {
        self.elevated
    }

    fn name(&self) -> &'static str {
        "windows"
    }
}

#[cfg(windows)]
mod imp {
    use super::{EnvVar, Scope, ValueKind};
    use crate::error::{AppError, AppResult};
    use winreg::enums::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
    };
    use winreg::{RegKey, RegValue};

    fn root(user: bool) -> RegKey {
        RegKey::predef(if user {
            HKEY_CURRENT_USER
        } else {
            HKEY_LOCAL_MACHINE
        })
    }

    pub fn decode_utf16(bytes: &[u8]) -> String {
        let words: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let end = words.iter().position(|&w| w == 0).unwrap_or(words.len());
        String::from_utf16_lossy(&words[..end])
    }

    pub fn encode_utf16(s: &str) -> Vec<u8> {
        s.encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(|w| w.to_le_bytes())
            .collect()
    }

    pub fn read_key(scope: Scope, user: bool, path: &str) -> AppResult<Vec<EnvVar>> {
        let key = match root(user).open_subkey_with_flags(path, KEY_READ) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let mut out = Vec::new();
        for item in key.enum_values() {
            let (name, val) = item?;
            let kind = match val.vtype {
                REG_SZ => ValueKind::String,
                REG_EXPAND_SZ => ValueKind::Expand,
                _ => ValueKind::Other,
            };
            let value = match kind {
                ValueKind::Other => String::new(),
                _ => decode_utf16(&val.bytes),
            };
            out.push(EnvVar {
                name,
                value,
                kind,
                scope,
            });
        }
        Ok(out)
    }

    pub fn write_value(
        user: bool,
        path: &str,
        name: &str,
        value: &str,
        kind: ValueKind,
    ) -> AppResult<()> {
        let vtype = match kind {
            ValueKind::String => REG_SZ,
            ValueKind::Expand => REG_EXPAND_SZ,
            ValueKind::Other => {
                return Err(AppError::Unsupported("bu değer türü yazılamaz".into()))
            }
        };
        let key = root(user).open_subkey_with_flags(path, KEY_SET_VALUE)?;
        key.set_raw_value(
            name,
            &RegValue {
                bytes: encode_utf16(value).into(),
                vtype,
            },
        )?;
        Ok(())
    }

    pub fn delete_value(user: bool, path: &str, name: &str) -> AppResult<()> {
        let key = root(user).open_subkey_with_flags(path, KEY_SET_VALUE)?;
        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn is_elevated() -> bool {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }
            let mut elev = TOKEN_ELEVATION::default();
            let mut len = 0u32;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elev as *mut _ as *mut core::ffi::c_void),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut len,
            )
            .is_ok();
            let _ = CloseHandle(token);
            ok && elev.TokenIsElevated != 0
        }
    }

    /// Açık pencerelere ortamın değiştiğini bildirir. Yanıt vermeyen pencereler en fazla
    /// 3 saniye bekletir; mesajı dinlemeyen uygulamalar eski ortamla çalışmaya devam eder.
    pub fn broadcast_environment_change() {
        use windows::core::w;
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };
        unsafe {
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(w!("Environment").as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                3000,
                None,
            );
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::{EnvVar, Scope, ValueKind};
    use crate::error::{AppError, AppResult};

    pub fn read_key(_: Scope, _: bool, _: &str) -> AppResult<Vec<EnvVar>> {
        Ok(Vec::new())
    }
    pub fn write_value(_: bool, _: &str, _: &str, _: &str, _: ValueKind) -> AppResult<()> {
        Err(AppError::Unsupported("yalnızca Windows".into()))
    }
    pub fn delete_value(_: bool, _: &str, _: &str) -> AppResult<()> {
        Err(AppError::Unsupported("yalnızca Windows".into()))
    }
    pub fn is_elevated() -> bool {
        false
    }
    pub fn broadcast_environment_change() {}
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// Salt okunur: gerçek kayıt defterini okur, hiçbir şey yazmaz.
    #[test]
    fn reads_real_scopes_without_modifying() {
        let p = WindowsProvider::new();
        let machine = p.list(Scope::Machine).unwrap();
        assert!(
            machine.iter().any(|v| v.name.eq_ignore_ascii_case("Path")),
            "sistem PATH okunabilmeli"
        );
        assert!(p.list(Scope::User).is_ok());
        let process = p.list(Scope::Process).unwrap();
        assert!(!process.is_empty());
        assert!(process.iter().all(|v| !v.name.starts_with('=')));
    }

    #[test]
    fn utf16_round_trip() {
        let s = "C:\\Kullanıcılar\\çğış;%USERPROFILE%";
        assert_eq!(imp::decode_utf16(&imp::encode_utf16(s)), s);
    }
}
