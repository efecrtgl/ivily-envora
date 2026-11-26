//! Pano işlemleri (Windows).
//!
//! Hassas değerler arayüze gönderilmeden doğrudan arka uçtan panoya kopyalanır. Bu durumda:
//!
//! - Windows'un tanımladığı `ExcludeClipboardContentFromMonitorProcessing`,
//!   `CanIncludeInClipboardHistory = 0` ve `CanUploadToCloudClipboard = 0` biçimleri eklenir;
//!   böylece değer pano geçmişine (Win+V) ve bulut panosuna alınmaz.
//! - Belirlenen süre sonunda pano, **yalnızca o arada başka bir şey kopyalanmadıysa** temizlenir
//!   (pano sıra numarası karşılaştırılır).
//!
//! Sınırlama: Panoyu sürekli izleyen üçüncü taraf yazılımlar değeri kopyalandığı anda okuyabilir.

use crate::error::{AppError, AppResult};

/// Metni panoya yazar ve yazma sonrası pano sıra numarasını döndürür.
pub fn copy_text(text: &str, sensitive: bool) -> AppResult<u32> {
    imp::copy_text(text, sensitive)
}

/// Pano sıra numarası değişmediyse panoyu boşaltır.
pub fn clear_if_unchanged(seq: u32) -> AppResult<bool> {
    imp::clear_if_unchanged(seq)
}

/// `seconds` saniye sonra panoyu (değişmediyse) temizleyen bir iş parçacığı başlatır.
pub fn schedule_clear(seq: u32, seconds: u32) {
    if seconds == 0 {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(seconds as u64));
        let _ = clear_if_unchanged(seq);
    });
}

#[cfg(windows)]
mod imp {
    use super::{AppError, AppResult};
    use windows::core::w;
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardSequenceNumber, OpenClipboard,
        RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    const CF_UNICODETEXT: u32 = 13;

    struct Open;
    impl Open {
        fn new() -> AppResult<Open> {
            // Başka bir uygulama panoyu kısa süreliğine tutuyor olabilir; birkaç kez dene.
            for _ in 0..10 {
                if unsafe { OpenClipboard(None) }.is_ok() {
                    return Ok(Open);
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(AppError::Platform("pano açılamadı".into()))
        }
    }
    impl Drop for Open {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseClipboard();
            }
        }
    }

    fn global_from_bytes(bytes: &[u8]) -> AppResult<HGLOBAL> {
        unsafe {
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1))
                .map_err(|e| AppError::Platform(format!("GlobalAlloc: {e}")))?;
            let p = GlobalLock(h) as *mut u8;
            if p.is_null() {
                let _ = GlobalFree(Some(h));
                return Err(AppError::Platform("GlobalLock başarısız".into()));
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
            let _ = GlobalUnlock(h);
            Ok(h)
        }
    }

    fn set(format: u32, bytes: &[u8]) -> AppResult<()> {
        let h = global_from_bytes(bytes)?;
        unsafe {
            if SetClipboardData(format, Some(HANDLE(h.0))).is_err() {
                // Başarısız olursa bellek hâlâ bizimdir.
                let _ = GlobalFree(Some(h));
                return Err(AppError::Platform("SetClipboardData başarısız".into()));
            }
        }
        Ok(())
    }

    pub fn copy_text(text: &str, sensitive: bool) -> AppResult<u32> {
        let utf16: Vec<u8> = text
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let _open = Open::new()?;
        unsafe {
            EmptyClipboard().map_err(|e| AppError::Platform(format!("EmptyClipboard: {e}")))?;
        }
        set(CF_UNICODETEXT, &utf16)?;
        if sensitive {
            unsafe {
                let exclude =
                    RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
                let history = RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"));
                let cloud = RegisterClipboardFormatW(w!("CanUploadToCloudClipboard"));
                if exclude != 0 {
                    let _ = set(exclude, &[0u8]);
                }
                let zero = 0u32.to_le_bytes();
                if history != 0 {
                    let _ = set(history, &zero);
                }
                if cloud != 0 {
                    let _ = set(cloud, &zero);
                }
            }
        }
        drop(_open);
        Ok(unsafe { GetClipboardSequenceNumber() })
    }

    pub fn clear_if_unchanged(seq: u32) -> AppResult<bool> {
        if unsafe { GetClipboardSequenceNumber() } != seq {
            return Ok(false);
        }
        let _open = Open::new()?;
        unsafe {
            EmptyClipboard().map_err(|e| AppError::Platform(format!("EmptyClipboard: {e}")))?;
        }
        Ok(true)
    }
}

#[cfg(not(windows))]
mod imp {
    use super::{AppError, AppResult};
    pub fn copy_text(_: &str, _: bool) -> AppResult<u32> {
        Err(AppError::Unsupported("yalnızca Windows".into()))
    }
    pub fn clear_if_unchanged(_: u32) -> AppResult<bool> {
        Ok(false)
    }
}
