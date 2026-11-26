//! Gizli değerler için Windows DPAPI kasası.
//!
//! Hassas olarak sınıflandırılan değerler (profil, proje, anlık görüntü) veritabanına
//! yazılmadan önce `CryptProtectData` ile şifrelenir.
//!
//! # Tehdit modeli ve sınırlamalar
//!
//! - `CurrentUser` kapsamı: Şifreli veri yalnızca aynı Windows kullanıcı hesabıyla çözülebilir.
//!   Veritabanı dosyasının başka bir kullanıcıya/makineye kopyalanması durumunda değerler korunur.
//! - Aynı kullanıcı olarak çalışan kötü amaçlı yazılımlar DPAPI ile şifrelenmiş veriyi çözebilir.
//!   DPAPI bu saldırgana karşı koruma **sağlamaz**.
//! - `LocalMachine` kapsamı yalnızca yükseltilmiş yardımcıya aktarılan, kısa ömürlü plan dosyası
//!   için kullanılır; makinedeki herhangi bir işlem bu veriyi çözebilir, bu yüzden dosya işlem
//!   tamamlanınca hemen silinir.
//! - Entropi sabiti gizli değildir; yalnızca başka uygulamaların DPAPI verileriyle karışmayı önler.

use crate::error::{AppError, AppResult};

/// Şifreli verinin başına eklenen biçim işareti. Biçim değişirse sürüm artırılır.
const MAGIC: &[u8; 4] = b"EVV1";
const ENTROPY: &[u8] = b"Ivily.Envora.Vault.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultScope {
    CurrentUser,
    LocalMachine,
}

pub fn protect(plain: &[u8], scope: VaultScope) -> AppResult<Vec<u8>> {
    let blob = imp::protect(plain, scope)?;
    let mut out = Vec::with_capacity(MAGIC.len() + blob.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&blob);
    Ok(out)
}

pub fn unprotect(data: &[u8]) -> AppResult<Vec<u8>> {
    let body = data
        .strip_prefix(MAGIC.as_slice())
        .ok_or_else(|| AppError::Vault("tanınmayan kasa biçimi".into()))?;
    imp::unprotect(body)
}

/// Metin değerini şifreler (kullanıcı kapsamı).
pub fn protect_str(value: &str) -> AppResult<Vec<u8>> {
    protect(value.as_bytes(), VaultScope::CurrentUser)
}

pub fn unprotect_str(data: &[u8]) -> AppResult<String> {
    String::from_utf8(unprotect(data)?)
        .map_err(|_| AppError::Vault("çözülen veri geçerli UTF-8 değil".into()))
}

#[cfg(windows)]
mod imp {
    use super::{VaultScope, ENTROPY};
    use crate::error::{AppError, AppResult};
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_LOCAL_MACHINE,
        CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }

    /// DPAPI'nin ayırdığı çıktı belleğini kopyalayıp serbest bırakır.
    unsafe fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData as _)));
        v
    }

    pub fn protect(plain: &[u8], scope: VaultScope) -> AppResult<Vec<u8>> {
        let input = blob(plain);
        let entropy = blob(ENTROPY);
        let mut out = CRYPT_INTEGER_BLOB::default();
        let mut flags = CRYPTPROTECT_UI_FORBIDDEN;
        if scope == VaultScope::LocalMachine {
            flags |= CRYPTPROTECT_LOCAL_MACHINE;
        }
        unsafe {
            CryptProtectData(&input, None, Some(&entropy), None, None, flags, &mut out)
                .map_err(|e| AppError::Vault(format!("CryptProtectData: {e}")))?;
            Ok(take(out))
        }
    }

    pub fn unprotect(data: &[u8]) -> AppResult<Vec<u8>> {
        let input = blob(data);
        let entropy = blob(ENTROPY);
        let mut out = CRYPT_INTEGER_BLOB::default();
        unsafe {
            CryptUnprotectData(
                &input,
                None,
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
            .map_err(|e| AppError::Vault(format!("CryptUnprotectData: {e}")))?;
            Ok(take(out))
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::VaultScope;
    use crate::error::{AppError, AppResult};

    pub fn protect(_plain: &[u8], _scope: VaultScope) -> AppResult<Vec<u8>> {
        Err(AppError::Unsupported(
            "güvenli depolama yalnızca Windows'ta kullanılabilir".into(),
        ))
    }

    pub fn unprotect(_data: &[u8]) -> AppResult<Vec<u8>> {
        Err(AppError::Unsupported(
            "güvenli depolama yalnızca Windows'ta kullanılabilir".into(),
        ))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn round_trip_user_scope() {
        let secret = "sk-test-not-a-real-key-ÇĞİÖŞÜ";
        let enc = protect_str(secret).unwrap();
        assert!(enc.starts_with(MAGIC));
        assert!(
            !enc.windows(secret.len()).any(|w| w == secret.as_bytes()),
            "şifreli veri düz metni içermemeli"
        );
        assert_eq!(unprotect_str(&enc).unwrap(), secret);
    }

    #[test]
    fn round_trip_machine_scope() {
        let enc = protect(b"plan", VaultScope::LocalMachine).unwrap();
        assert_eq!(unprotect(&enc).unwrap(), b"plan");
    }

    #[test]
    fn rejects_tampered_or_foreign_data() {
        let mut enc = protect_str("degisken").unwrap();
        let last = enc.len() - 1;
        enc[last] ^= 0xff;
        assert!(unprotect(&enc).is_err());
        assert!(unprotect(b"plaintext").is_err());
    }

    #[test]
    fn empty_value_round_trips() {
        let enc = protect_str("").unwrap();
        assert_eq!(unprotect_str(&enc).unwrap(), "");
    }
}
