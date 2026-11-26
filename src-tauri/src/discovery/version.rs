//! Sürüm bilgisi: dosya kaynağı, package.json ve (son çare) sınırlı `--version` sorgusu.

use crate::env::effective::EnvMap;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// npm/pnpm/yarn gibi `.cmd` araçlar: `<dizin>\node_modules\<ad>\package.json` içindeki `version`.
pub fn from_package(cmd_path: &Path, name: &str) -> Option<String> {
    let dir = cmd_path.parent()?;
    let candidates = [
        dir.join("node_modules").join(name).join("package.json"),
        dir.join("node_modules")
            .join(format!("{name}-cli"))
            .join("package.json"),
    ];
    for c in candidates {
        let Ok(text) = std::fs::read_to_string(&c) else {
            continue;
        };
        if text.len() > 512 * 1024 {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(&text).ok()?;
        if let Some(ver) = v.get("version").and_then(|x| x.as_str()) {
            if ver.len() <= 64 {
                return Some(ver.to_string());
            }
        }
    }
    None
}

/// Yürütülebilir dosyanın sürüm kaynağından ürün sürümü (dosya çalıştırılmaz).
pub fn from_resource(path: &Path) -> Option<String> {
    imp::product_version(path)
}

/// Sürüm çıktısından ilk anlamlı satırı ayıklar ve kısaltır.
pub fn first_line(out: &str) -> Option<String> {
    out.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.chars().take(120).collect())
}

/// `--version` sorgusu: sabit argümanlar, girdi yok, pencere yok, zaman aşımı, çıktı sınırlı.
pub fn probe(exe: &Path, args: &[&str], env: &EnvMap, timeout: Duration) -> Option<String> {
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args)
        .env_clear()
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (name, value) in env.values() {
        cmd.env(name, value);
    }
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().ok()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
    let read = |r: Option<&mut dyn Read>| -> String {
        let mut buf = Vec::new();
        if let Some(r) = r {
            let _ = r.take(4096).read_to_end(&mut buf);
        }
        String::from_utf8_lossy(&buf).into_owned()
    };
    let out = read(child.stdout.as_mut().map(|s| s as &mut dyn Read));
    let err = read(child.stderr.as_mut().map(|s| s as &mut dyn Read));
    // Bazı araçlar (ör. java -version) sürümü stderr'e yazar.
    first_line(&out).or_else(|| first_line(&err))
}

#[cfg(windows)]
mod imp {
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };

    pub fn product_version(path: &Path) -> Option<String> {
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            let size = GetFileVersionInfoSizeW(PCWSTR(wide.as_ptr()), None);
            if size == 0 || size > 4 * 1024 * 1024 {
                return None;
            }
            let mut buf = vec![0u8; size as usize];
            GetFileVersionInfoW(
                PCWSTR(wide.as_ptr()),
                None,
                size,
                buf.as_mut_ptr() as *mut _,
            )
            .ok()?;
            let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
            let mut len = 0u32;
            if !VerQueryValueW(buf.as_ptr() as *const _, w!("\\"), &mut ptr, &mut len).as_bool()
                || ptr.is_null()
            {
                return None;
            }
            if (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>() {
                return None;
            }
            let info = &*(ptr as *const VS_FIXEDFILEINFO);
            if info.dwSignature != 0xFEEF04BD {
                return None;
            }
            let (ms, ls) = (info.dwProductVersionMS, info.dwProductVersionLS);
            let parts = [ms >> 16, ms & 0xffff, ls >> 16, ls & 0xffff];
            // Tümü sıfırsa sürüm kaynağı anlamsızdır.
            if parts.iter().all(|p| *p == 0) {
                return None;
            }
            // Sondaki sıfırları kırp ama en az üç parça bırak (ör. 24.15.0.0 → 24.15.0).
            let mut end = 4;
            while end > 3 && parts[end - 1] == 0 {
                end -= 1;
            }
            Some(
                parts[..end]
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn product_version(_: &std::path::Path) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_version_is_read_without_running() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("npm.cmd"), "@echo should never run").unwrap();
        let pkg = d.path().join("node_modules").join("npm");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"npm","version":"10.9.0"}"#,
        )
        .unwrap();
        assert_eq!(
            from_package(&d.path().join("npm.cmd"), "npm").as_deref(),
            Some("10.9.0")
        );
        assert_eq!(from_package(&d.path().join("yarn.cmd"), "yarn"), None);
    }

    #[test]
    fn first_line_trims() {
        assert_eq!(
            first_line("\n  v22.1.0  \nextra").as_deref(),
            Some("v22.1.0")
        );
        assert_eq!(first_line(""), None);
    }

    #[cfg(windows)]
    #[test]
    fn reads_file_resource_of_system_binary() {
        let cmd = std::path::PathBuf::from(
            std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()),
        )
        .join("System32\\cmd.exe");
        assert!(from_resource(&cmd).is_some());
    }
}
