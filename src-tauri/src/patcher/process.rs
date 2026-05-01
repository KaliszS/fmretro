use std::ffi::c_void;
use std::mem::{size_of, zeroed};

use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, Process32FirstW, Process32NextW,
    MODULEENTRY32W, PROCESSENTRY32W, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, VirtualProtectEx, VirtualQueryEx, MEMORY_BASIC_INFORMATION,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::System::ProcessStatus::{K32EnumProcessModules, K32GetModuleBaseNameW, K32GetModuleInformation, MODULEINFO};

use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};

#[derive(Debug, thiserror::Error)]
pub enum ProcError {
    #[error("Win32 error: {0}")]
    Win(#[from] windows::core::Error),
    #[allow(dead_code)]
    #[error("fm.exe not found")]
    NotFound,
    #[error("module not found: {0}")]
    Module(String),
    #[error("io: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, ProcError>;

#[derive(Clone, Copy)]
pub struct ModuleInfo {
    pub base: usize,
    pub size: usize,
}

pub struct FmProcess {
    pub pid: u32,
    pub handle: HANDLE,
}

impl Drop for FmProcess {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

unsafe impl Send for FmProcess {}
unsafe impl Sync for FmProcess {}

impl FmProcess {
    pub fn find_pid(name: &str) -> Option<u32> {
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: size_of::<PROCESSENTRY32W>() as u32,
                ..zeroed()
            };
            let mut found = None;
            if Process32FirstW(snap, &mut entry).is_ok() {
                loop {
                    let exe = String::from_utf16_lossy(
                        &entry.szExeFile[..entry
                            .szExeFile
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(entry.szExeFile.len())],
                    );
                    if exe.eq_ignore_ascii_case(name) {
                        found = Some(entry.th32ProcessID);
                        break;
                    }
                    if Process32NextW(snap, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snap);
            found
        }
    }

    pub fn open(pid: u32) -> Result<Self> {
        unsafe {
            let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
            Ok(Self { pid, handle })
        }
    }

    pub fn read(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        let mut got = 0usize;
        unsafe {
            ReadProcessMemory(
                self.handle,
                addr as *const c_void,
                buf.as_mut_ptr() as *mut c_void,
                size,
                Some(&mut got),
            )
            .map_err(|e| ProcError::Io(format!("read 0x{:X}: {}", addr, e)))?;
        }
        buf.truncate(got);
        Ok(buf)
    }

    pub fn write(&self, addr: usize, data: &[u8]) -> Result<()> {
        let mut written = 0usize;
        unsafe {
            WriteProcessMemory(
                self.handle,
                addr as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut written),
            )
            .map_err(|e| ProcError::Io(format!("write 0x{:X}: {}", addr, e)))?;
        }
        Ok(())
    }

    pub fn read_ascii_digits(&self, addr: usize, max: usize) -> Result<String> {
        let raw = self.read(addr, max)?;
        let mut s = String::new();
        for &b in &raw {
            if (0x30..=0x39).contains(&b) {
                s.push(b as char);
            } else {
                break;
            }
        }
        Ok(s)
    }

    pub fn write_ascii_num(&self, addr: usize, value: &str) -> Result<()> {
        // Zero 4 bytes then write ASCII (matches Python behavior).
        self.write(addr, &[0, 0, 0, 0])?;
        self.write(addr, value.as_bytes())?;
        Ok(())
    }

    /// Scan committed RW pages for a byte pattern.
    pub fn scan(&self, pattern: &[u8]) -> Option<usize> {
        let mut addr: usize = 0;
        let limit: usize = 0x7FFF_FFFF_FFFF;
        let prot_rw = 0x04 | 0x08 | 0x40 | 0x80;
        let prot_guard = 0x100u32;
        unsafe {
            while addr < limit {
                let mut mbi: MEMORY_BASIC_INFORMATION = zeroed();
                let r = VirtualQueryEx(
                    self.handle,
                    Some(addr as *const c_void),
                    &mut mbi,
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                );
                if r == 0 {
                    addr += 0x1000;
                    continue;
                }
                let prot = mbi.Protect.0;
                let state = mbi.State.0;
                if state == MEM_COMMIT.0
                    && (prot & prot_rw) != 0
                    && (prot & prot_guard) == 0
                {
                    let region_size = mbi.RegionSize;
                    let base = mbi.BaseAddress as usize;
                    if let Ok(data) = self.read(base, region_size) {
                        if let Some(idx) = find_pattern(&data, pattern) {
                            return Some(base + idx);
                        }
                    }
                }
                addr = mbi.BaseAddress as usize + mbi.RegionSize;
            }
        }
        None
    }

    pub fn module(&self, name: &str) -> Result<ModuleInfo> {
        unsafe {
            let mut needed = 0u32;
            let _ = K32EnumProcessModules(self.handle, std::ptr::null_mut(), 0, &mut needed);
            let count = (needed as usize) / size_of::<HMODULE>();
            let mut mods: Vec<HMODULE> = vec![HMODULE::default(); count.max(1)];
            let cb = (count * size_of::<HMODULE>()) as u32;
            K32EnumProcessModules(self.handle, mods.as_mut_ptr(), cb, &mut needed)
                .ok()
                .map_err(|e| ProcError::Io(e.to_string()))?;
            for m in mods.iter().take(count) {
                let mut buf = [0u16; 260];
                let n = K32GetModuleBaseNameW(self.handle, *m, &mut buf);
                if n == 0 {
                    continue;
                }
                let s = String::from_utf16_lossy(&buf[..n as usize]);
                if s.eq_ignore_ascii_case(name) {
                    let mut info: MODULEINFO = zeroed();
                    K32GetModuleInformation(
                        self.handle,
                        *m,
                        &mut info,
                        size_of::<MODULEINFO>() as u32,
                    )
                    .ok()
                    .map_err(|e| ProcError::Io(e.to_string()))?;
                    return Ok(ModuleInfo {
                        base: info.lpBaseOfDll as usize,
                        size: info.SizeOfImage as usize,
                    });
                }
            }
        }
        Err(ProcError::Module(name.to_string()))
    }

    pub fn alloc_near(&self, target: usize, size: usize) -> Option<usize> {
        let gran: usize = 0x10000;
        let radius: usize = 0x7000_0000;
        for direction in [-1isize, 1isize] {
            let aligned = target & !(gran - 1);
            let mut addr = if direction == -1 {
                aligned.wrapping_sub(gran)
            } else {
                aligned.wrapping_add(gran)
            };
            let limit = if direction == -1 {
                target.saturating_sub(radius)
            } else {
                target.saturating_add(radius)
            };
            loop {
                let cont = if direction == -1 { addr >= limit } else { addr <= limit };
                if !cont {
                    break;
                }
                unsafe {
                    let p = VirtualAllocEx(
                        self.handle,
                        Some(addr as *const c_void),
                        size,
                        MEM_COMMIT | MEM_RESERVE,
                        PAGE_EXECUTE_READWRITE,
                    );
                    if !p.is_null() {
                        return Some(p as usize);
                    }
                }
                if direction == -1 {
                    if addr < gran {
                        break;
                    }
                    addr -= gran;
                } else {
                    addr = addr.wrapping_add(gran);
                }
            }
        }
        None
    }

    pub fn free(&self, addr: usize) {
        unsafe {
            let _ = VirtualFreeEx(self.handle, addr as *mut c_void, 0, MEM_RELEASE);
        }
    }

    /// Write to potentially read-only page using VirtualProtectEx.
    pub fn vwrite(&self, addr: usize, data: &[u8]) -> Result<()> {
        unsafe {
            let mut old = PAGE_PROTECTION_FLAGS(0);
            VirtualProtectEx(
                self.handle,
                addr as *const c_void,
                data.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old,
            )
            .map_err(|e| ProcError::Io(format!("VirtualProtectEx: {}", e)))?;
            let res = self.write(addr, data);
            let mut tmp = PAGE_PROTECTION_FLAGS(0);
            let _ = VirtualProtectEx(self.handle, addr as *const c_void, data.len(), old, &mut tmp);
            res
        }
    }
}

fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Helper for module enumeration via toolhelp (alternative path).
#[allow(dead_code)]
pub fn enum_module_toolhelp(pid: u32, name: &str) -> Option<ModuleInfo> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid).ok()?;
        let mut e = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..zeroed()
        };
        let mut found = None;
        if Module32FirstW(snap, &mut e).is_ok() {
            loop {
                let n = e.szModule.iter().position(|&c| c == 0).unwrap_or(0);
                let s = String::from_utf16_lossy(&e.szModule[..n]);
                if s.eq_ignore_ascii_case(name) {
                    found = Some(ModuleInfo {
                        base: e.modBaseAddr as usize,
                        size: e.modBaseSize as usize,
                    });
                    break;
                }
                if Module32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        found
    }
}
