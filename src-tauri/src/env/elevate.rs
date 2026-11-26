//! Sistem kapsamı için yükseltilmiş yardımcı.
//!
//! Uygulama standart kullanıcı yetkisiyle çalışır. Sistem kapsamındaki onaylanmış bir plan,
//! aynı yürütülebilir dosyanın `--envora-elevated-apply <plan> <sonuç>` argümanlarıyla `runas`
//! fiiliyle (UAC) başlatılmasıyla uygulanır:
//!
//! 1. Plan JSON olarak serileştirilir, DPAPI (`LocalMachine`) ile şifrelenip veri dizinindeki
//!    `elevate` klasörüne yazılır. Dosya yalnızca işlem süresince vardır.
//! 2. Yardımcı kip arayüz açmaz; planı çözer, şemayı ve tüm ad/değerleri yeniden doğrular,
//!    **yalnızca sistem kapsamındaki ortam değişkeni ayarlama/silme** işlemlerini uygular,
//!    sonucu (değer içermeyen) bir JSON dosyasına yazar ve çıkar.
//! 3. Ana süreç sonucu okur ve her iki dosyayı da siler.
//!
//! Yardımcı hiçbir komut, betik veya kabuk çalıştırmaz.

use super::apply::apply_ops;
use super::model::Scope;
use super::plan::Resolved;
use super::provider::EnvProvider;
use super::validate::{validate_name, validate_value};
use crate::error::{AppError, AppResult};
use crate::security::vault::{self, VaultScope};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const HELPER_FLAG: &str = "--envora-elevated-apply";
const FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct PlanFile {
    version: u32,
    ops: Vec<Resolved>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HelperResult {
    pub ok: bool,
    pub applied: usize,
    pub code: Option<String>,
    pub message: Option<String>,
}

fn check_ops(ops: &[Resolved]) -> AppResult<()> {
    if ops.is_empty() || ops.len() > super::plan::MAX_OPS {
        return Err(AppError::validation("plan_too_large", ""));
    }
    for op in ops {
        validate_name(op.name())?;
        if let Resolved::Set { value, kind, .. } = op {
            validate_value(value)?;
            if *kind == super::model::ValueKind::Other {
                return Err(AppError::validation("invalid_kind", ""));
            }
        }
    }
    Ok(())
}

pub fn encode_plan(ops: &[Resolved]) -> AppResult<Vec<u8>> {
    check_ops(ops)?;
    let json = serde_json::to_vec(&PlanFile {
        version: FORMAT_VERSION,
        ops: ops.to_vec(),
    })
    .map_err(|e| AppError::Internal(e.to_string()))?;
    vault::protect(&json, VaultScope::LocalMachine)
}

pub fn decode_plan(bytes: &[u8]) -> AppResult<Vec<Resolved>> {
    let json = vault::unprotect(bytes)?;
    let plan: PlanFile = serde_json::from_slice(&json)?;
    if plan.version != FORMAT_VERSION {
        return Err(AppError::validation(
            "unsupported_version",
            plan.version.to_string(),
        ));
    }
    check_ops(&plan.ops)?;
    Ok(plan.ops)
}

/// Yardımcı kipin asıl işi (test edilebilir): planı çöz, doğrula, sistem kapsamına uygula.
pub fn run_helper(p: &dyn EnvProvider, plan_bytes: &[u8]) -> HelperResult {
    let res = decode_plan(plan_bytes).and_then(|ops| {
        let n = apply_ops(p, Scope::Machine, &ops)?;
        p.broadcast_change();
        Ok(n)
    });
    match res {
        Ok(applied) => HelperResult {
            ok: true,
            applied,
            code: None,
            message: None,
        },
        Err(e) => HelperResult {
            ok: false,
            applied: 0,
            code: Some(e.code().to_string()),
            message: Some(e.to_string()),
        },
    }
}

/// `main` tarafından çağrılır. Yardımcı kip değilse `None` döner.
pub fn maybe_run_helper_from_args() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some(HELPER_FLAG) {
        return None;
    }
    let (Some(plan_path), Some(result_path)) = (args.get(2), args.get(3)) else {
        return Some(2);
    };
    let plan_path = PathBuf::from(plan_path);
    let result_path = PathBuf::from(result_path);
    let allowed = crate::state::data_dir()
        .map(|d| d.join("elevate"))
        .unwrap_or_default();
    if !paths_allowed(&allowed, &plan_path, &result_path) {
        return Some(3);
    }
    let result = match std::fs::read(&plan_path) {
        Ok(bytes) => {
            let _ = std::fs::remove_file(&plan_path);
            run_helper(&super::windows::WindowsProvider::new(), &bytes)
        }
        Err(e) => HelperResult {
            ok: false,
            applied: 0,
            code: Some("io".into()),
            message: Some(e.to_string()),
        },
    };
    let ok = result.ok;
    let _ = std::fs::write(
        &result_path,
        serde_json::to_vec(&result).unwrap_or_default(),
    );
    Some(if ok { 0 } else { 1 })
}

/// Plan ve sonuç dosyaları yalnızca uygulamanın `elevate` klasöründe olabilir; yardımcı başka
/// bir konumdaki dosyayı okumayı veya oraya yazmayı reddeder.
pub fn paths_allowed(dir: &Path, plan: &Path, result: &Path) -> bool {
    let Ok(dir) = std::fs::canonicalize(dir) else {
        return false;
    };
    let inside = |p: &Path| -> bool {
        // Sonuç dosyası henüz yok; üst dizini kanonikleştir.
        let parent = match p.parent() {
            Some(x) => x,
            None => return false,
        };
        let name_ok = p.file_name().is_some_and(|n| {
            let n = n.to_string_lossy();
            !n.contains("..") && (n.ends_with(".plan") || n.ends_with(".result.json"))
        });
        name_ok && std::fs::canonicalize(parent).is_ok_and(|c| c == dir)
    };
    inside(plan) && inside(result)
}

fn unique_name() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}-{nanos}", std::process::id())
}

/// Ana süreçten çağrılır: planı yazar, yardımcıyı UAC ile başlatır, sonucu okur.
pub fn run_elevated(data_dir: &Path, ops: &[Resolved]) -> AppResult<usize> {
    let dir = data_dir.join("elevate");
    std::fs::create_dir_all(&dir)?;
    let id = unique_name();
    let plan_path = dir.join(format!("{id}.plan"));
    let result_path = dir.join(format!("{id}.result.json"));
    std::fs::write(&plan_path, encode_plan(ops)?)?;

    let launched = imp::launch_and_wait(&plan_path, &result_path);
    // Yardımcı planı okuduğunda siler; başlatılamadıysa burada silinir.
    let _ = std::fs::remove_file(&plan_path);
    launched?;

    let raw = std::fs::read(&result_path)
        .map_err(|_| AppError::Platform("yükseltilmiş yardımcı sonuç üretmedi".into()));
    let _ = std::fs::remove_file(&result_path);
    let result: HelperResult = serde_json::from_slice(&raw?)?;
    if result.ok {
        Ok(result.applied)
    } else {
        let msg = result.message.unwrap_or_default();
        Err(match result.code.as_deref() {
            Some("permission_denied") => AppError::PermissionDenied(msg),
            Some(code) if code.starts_with("invalid") => AppError::validation("invalid_plan", msg),
            _ => AppError::Platform(msg),
        })
    }
}

#[cfg(windows)]
mod imp {
    use crate::error::{AppError, AppResult};
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    fn quote(p: &Path) -> String {
        format!("\"{}\"", p.display())
    }

    pub fn launch_and_wait(plan: &Path, result: &Path) -> AppResult<()> {
        let exe = std::env::current_exe()?;
        let exe_w = wide(exe.as_os_str());
        let params = format!("{} {} {}", super::HELPER_FLAG, quote(plan), quote(result));
        let params_w = wide(std::ffi::OsStr::new(&params));
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS,
            lpVerb: w!("runas"),
            lpFile: PCWSTR(exe_w.as_ptr()),
            lpParameters: PCWSTR(params_w.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };
        unsafe {
            if let Err(e) = ShellExecuteExW(&mut info) {
                if e.code() == ERROR_CANCELLED.to_hresult() {
                    return Err(AppError::Cancelled);
                }
                return Err(AppError::Platform(format!("yükseltme başlatılamadı: {e}")));
            }
            if info.hProcess.is_invalid() {
                return Err(AppError::Platform(
                    "yardımcı süreç tanıtıcısı alınamadı".into(),
                ));
            }
            let wait = WaitForSingleObject(info.hProcess, 5 * 60 * 1000);
            let mut code = 0u32;
            let _ = GetExitCodeProcess(info.hProcess, &mut code);
            let _ = CloseHandle(info.hProcess);
            if wait != WAIT_OBJECT_0 {
                return Err(AppError::Platform(
                    "yükseltilmiş yardımcı zaman aşımına uğradı".into(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::error::{AppError, AppResult};
    use std::path::Path;
    pub fn launch_and_wait(_: &Path, _: &Path) -> AppResult<()> {
        Err(AppError::Unsupported("yalnızca Windows".into()))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::env::model::ValueKind;
    use crate::env::provider::MemoryProvider;

    #[test]
    fn helper_applies_only_to_machine_scope() {
        let p = MemoryProvider::new().with_machine_write(true);
        p.seed(Scope::User, "X", "user-value", ValueKind::String);
        let ops = vec![Resolved::Set {
            name: "X".into(),
            value: "machine-value".into(),
            kind: ValueKind::String,
        }];
        let bytes = encode_plan(&ops).unwrap();
        let r = run_helper(&p, &bytes);
        assert!(r.ok, "{r:?}");
        assert_eq!(
            p.get(Scope::Machine, "X").unwrap().unwrap().value,
            "machine-value"
        );
        assert_eq!(
            p.get(Scope::User, "X").unwrap().unwrap().value,
            "user-value",
            "kullanıcı kapsamı etkilenmemeli"
        );
    }

    #[test]
    fn helper_rejects_tampered_or_foreign_plans() {
        let p = MemoryProvider::new().with_machine_write(true);
        assert!(
            !run_helper(&p, b"{\"version\":1,\"ops\":[]}").ok,
            "şifresiz plan reddedilmeli"
        );
        let mut bytes = encode_plan(&[Resolved::Delete { name: "X".into() }]).unwrap();
        let n = bytes.len();
        bytes[n - 1] ^= 1;
        assert!(!run_helper(&p, &bytes).ok);
        let bad = vault::protect(
            br#"{"version":1,"ops":[{"op":"set","name":"A=B","value":"1","kind":"string"}]}"#,
            VaultScope::LocalMachine,
        )
        .unwrap();
        let r = run_helper(&p, &bad);
        assert!(!r.ok && r.code.as_deref() == Some("invalid_name_equals"));
        let unknown = vault::protect(
            br#"{"version":1,"ops":[{"op":"exec","cmd":"calc"}]}"#,
            VaultScope::LocalMachine,
        )
        .unwrap();
        assert!(
            !run_helper(&p, &unknown).ok,
            "bilinmeyen işlem türü reddedilmeli"
        );
    }

    #[test]
    fn helper_only_accepts_files_in_elevate_dir() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("elevate");
        std::fs::create_dir_all(&dir).unwrap();
        let other = root.path().join("other");
        std::fs::create_dir_all(&other).unwrap();
        assert!(paths_allowed(
            &dir,
            &dir.join("1.plan"),
            &dir.join("1.result.json")
        ));
        assert!(!paths_allowed(
            &dir,
            &other.join("1.plan"),
            &dir.join("1.result.json")
        ));
        assert!(!paths_allowed(
            &dir,
            &dir.join("1.plan"),
            &other.join("x.result.json")
        ));
        assert!(!paths_allowed(
            &dir,
            &dir.join("..").join("other").join("1.plan"),
            &dir.join("1.result.json")
        ));
        assert!(!paths_allowed(
            &dir,
            &dir.join("1.txt"),
            &dir.join("1.result.json")
        ));
    }

    #[test]
    fn result_does_not_contain_values() {
        let p = MemoryProvider::new(); // sistem yazma izni yok → başarısız
        let ops = vec![Resolved::Set {
            name: "TOKEN".into(),
            value: "very-secret-value".into(),
            kind: ValueKind::String,
        }];
        let r = run_helper(&p, &encode_plan(&ops).unwrap());
        assert!(!r.ok);
        assert!(!serde_json::to_string(&r).unwrap().contains("very-secret"));
    }
}
