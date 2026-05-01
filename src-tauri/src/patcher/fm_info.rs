use std::ffi::c_void;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

/// Major.minor of the FM executable on disk for the given PID.
/// Accepts any FM major version (no hard-coded year range): tries PE version
/// info first, then falls back to parsing the install folder name
/// (e.g. "Football Manager 2024" → 24).
pub fn fm_version(pid: u32) -> Option<(u16, u16)> {
    let path = process_path(pid)?;

    if let Some(v) = pe_version(&path) {
        // Treat anything outside a sane FM-like range as bogus and try the folder.
        if v.0 > 0 && v.0 < 100 {
            return Some(v);
        }
    }
    folder_year(&path).map(|m| (m, 0))
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let path = module_path(h);
        let _ = CloseHandle(h);
        path
    }
}

/// Public wrapper to expose the executable path for debug display.
pub fn fm_path(pid: u32) -> Option<String> {
    process_path(pid)
}

fn pe_version(path: &str) -> Option<(u16, u16)> {
    unsafe {
        let mut wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let pcw = PCWSTR(wide.as_mut_ptr());

        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(pcw, Some(&mut handle));
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        GetFileVersionInfoW(pcw, 0, size, buf.as_mut_ptr() as *mut c_void).ok()?;

        let mut info_ptr: *mut c_void = std::ptr::null_mut();
        let mut info_len: u32 = 0;
        let ok = VerQueryValueW(
            buf.as_ptr() as *const c_void,
            w!("\\"),
            &mut info_ptr,
            &mut info_len,
        )
        .as_bool();
        if !ok || info_ptr.is_null() {
            return None;
        }
        let ffi = &*(info_ptr as *const VS_FIXEDFILEINFO);
        let major = (ffi.dwFileVersionMS >> 16) as u16;
        let minor = (ffi.dwFileVersionMS & 0xFFFF) as u16;
        Some((major, minor))
    }
}

/// Parse "...\Football Manager 2024\fm.exe" → 24.
fn folder_year(path: &str) -> Option<u16> {
    let lower = path.to_ascii_lowercase();
    let key = "football manager ";
    let mut idx = 0usize;
    while let Some(pos) = lower[idx..].find(key) {
        let start = idx + pos + key.len();
        let digits: String = lower[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(year) = digits.parse::<u32>() {
            // Accept "Football Manager 2024" → 24 and "Football Manager 24" → 24.
            let yy = if year >= 2000 { year - 2000 } else { year };
            if yy > 0 && yy < 100 {
                return Some(yy as u16);
            }
        }
        idx = start;
    }
    None
}

/// Build a label for any FM major (no hard-coded range).
pub fn edition_label(major: u16) -> Option<(String, i32)> {
    if major == 0 || major >= 100 {
        return None;
    }
    let yy = 2000 + major as i32;
    Some((format!("FM{:02}", major % 100), yy - 1))
}

unsafe fn module_path(h: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let n = K32GetModuleFileNameExW(h, HMODULE::default(), &mut buf);
    if n == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buf[..n as usize]))
}
