use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;

use super::fm_info;
use super::hooks::{VcrtCareerHook, VcrtDWordHook, VcrtStrHook};
use super::process::FmProcess;
use super::string_table::{StringTable, ENTRY_STRIDE, YEAR_COUNT, YEAR_FIRST, YEAR_LAST};
use super::year_patcher::YearPatcher;

#[derive(Default)]
struct Hooks {
    str_hook: Option<VcrtStrHook>,
    career_hook: Option<VcrtCareerHook>,
    dword_hook: Option<VcrtDWordHook>,
}

#[derive(Default)]
struct Inner {
    proc: Option<Arc<FmProcess>>,
    table: Option<StringTable>,
    year_patcher: YearPatcher,
    hooks: Hooks,
    last_message: String,
}

#[derive(Default, Clone)]
pub struct PatchManager {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Serialize, Clone)]
pub struct FmEditionInfo {
    pub major: u16,
    pub minor: u16,
    pub label: String,
    pub start_year: i32,
}

#[derive(Serialize, Clone)]
pub struct PatchStatus {
    pub fm_detected: bool,
    pub pid: Option<u32>,
    pub patch_active: bool,
    pub shift: Option<i32>,
    pub min_year: Option<i32>,
    pub message: String,
    pub year_table_base: Option<String>,
    pub hooks: Vec<String>,
    pub edition: Option<FmEditionInfo>,
}

impl PatchManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn detect(&self) -> Option<u32> {
        // Reuse the existing handle if its PID still matches; otherwise re-open.
        let mut inner = self.inner.lock();
        let live_pid = FmProcess::find_pid("fm.exe");
        match (&inner.proc, live_pid) {
            (Some(p), Some(pid)) if p.pid == pid => Some(pid),
            (_, Some(pid)) => {
                // Drop stale handle (also drops any stale hook references — fm.exe gone).
                if inner.proc.as_ref().map(|p| p.pid) != Some(pid) {
                    inner.proc = None;
                    inner.table = None;
                    inner.year_patcher = YearPatcher::new();
                    inner.hooks = Hooks::default();
                }
                match FmProcess::open(pid) {
                    Ok(fp) => {
                        inner.proc = Some(Arc::new(fp));
                        Some(pid)
                    }
                    Err(_) => None,
                }
            }
            (_, None) => {
                inner.proc = None;
                inner.table = None;
                inner.year_patcher = YearPatcher::new();
                inner.hooks = Hooks::default();
                None
            }
        }
    }

    pub fn status(&self) -> PatchStatus {
        let inner = self.inner.lock();
        let mut hooks = Vec::new();
        if inner.hooks.str_hook.is_some() {
            hooks.push("Season suffix (YYYY/YY)".into());
        }
        if inner.hooks.career_hook.is_some() {
            hooks.push("Career range (YY-YY)".into());
        }
        if inner.hooks.dword_hook.is_some() {
            hooks.push("Year DWORD restoration".into());
        }
        PatchStatus {
            fm_detected: inner.proc.is_some(),
            pid: inner.proc.as_ref().map(|p| p.pid),
            patch_active: inner.year_patcher.shift_applied.is_some(),
            shift: inner.year_patcher.shift_applied,
            min_year: None,
            message: inner.last_message.clone(),
            year_table_base: inner.table.as_ref().map(|t| format!("0x{:X}", t.year_base)),
            hooks,
            edition: inner.proc.as_ref().and_then(|p| {
                let (major, minor) = fm_info::fm_version(p.pid)?;
                let (label, start_year) = fm_info::edition_label(major)?;
                Some(FmEditionInfo {
                    major,
                    minor,
                    label,
                    start_year,
                })
            }),
        }
    }

    pub fn apply(&self, shift_back: i32, min_year: i32) -> Result<String, String> {
        if shift_back <= 0 {
            return Err("shift_back must be positive".into());
        }
        let mut inner = self.inner.lock();
        if inner.year_patcher.shift_applied.is_some() {
            return Err("Patch already active. Restore first.".into());
        }
        let proc = inner
            .proc
            .clone()
            .ok_or_else(|| "fm.exe not connected".to_string())?;

        // Locate string table.
        let table = StringTable::locate(&proc).map_err(|e| e.to_string())?;
        let year_base = table.year_base;

        let shift = -shift_back;
        let min_y = if min_year < 0 { 0 } else { min_year };

        let n = inner
            .year_patcher
            .patch(&proc, &table, shift, min_y)
            .map_err(|e| e.to_string())?;
        inner.table = Some(table);

        // Install hooks.
        let mut messages = vec![format!("Shifted {} year entries (shift {:+})", n, shift)];

        match VcrtStrHook::install(&proc, shift_back, min_y, year_base) {
            Ok(h) => {
                inner.hooks.str_hook = Some(h);
                messages.push("Season suffix hook installed".into());
            }
            Err(e) => {
                // Roll back years.
                inner.year_patcher.restore(&proc);
                inner.table = None;
                return Err(format!("Season hook failed: {}", e));
            }
        }

        if let Ok(h) = VcrtCareerHook::install(&proc, shift_back, 0, year_base) {
            inner.hooks.career_hook = Some(h);
            messages.push("Career range hook installed".into());
        } else {
            messages.push("Career hook skipped (pattern not found)".into());
        }

        if min_y > 0 {
            if let Ok(h) = VcrtDWordHook::install(&proc, shift_back, min_y, year_base) {
                inner.hooks.dword_hook = Some(h);
                messages.push("Year DWORD restoration hook installed".into());
            } else {
                messages.push("DWord hook skipped".into());
            }
        }

        let summary = messages.join("; ");
        inner.last_message = summary.clone();
        Ok(summary)
    }

    pub fn restore(&self) -> Result<String, String> {
        let mut inner = self.inner.lock();
        let proc = match inner.proc.clone() {
            Some(p) => p,
            None => {
                inner.last_message = "fm.exe not connected".into();
                return Ok(inner.last_message.clone());
            }
        };
        let mut parts = Vec::new();
        if let Some(h) = inner.hooks.str_hook.take() {
            h.remove(&proc);
            parts.push("season hook removed");
        }
        if let Some(h) = inner.hooks.career_hook.take() {
            h.remove(&proc);
            parts.push("career hook removed");
        }
        if let Some(h) = inner.hooks.dword_hook.take() {
            h.remove(&proc);
            parts.push("dword hook removed");
        }
        let restored = inner.year_patcher.restore(&proc);
        if restored > 0 {
            parts.push("years restored");
        }
        let msg = if parts.is_empty() {
            "Nothing to restore".to_string()
        } else {
            parts.join(", ")
        };
        inner.last_message = msg.clone();
        Ok(msg)
    }

    pub fn debug(&self) -> DebugInfo {
        let inner = self.inner.lock();

        let process = inner.proc.as_ref().map(|p| ProcessDebug {
            pid: p.pid,
            path: fm_info::fm_path(p.pid),
            fm_module: p.module("fm.exe").ok().map(|m| ModuleDebug {
                base: format!("0x{:X}", m.base),
                size: m.size as u64,
                size_mb: (m.size as f64) / (1024.0 * 1024.0),
            }),
            vcrt_module: p.module("VCRUNTIME140.dll").ok().map(|m| ModuleDebug {
                base: format!("0x{:X}", m.base),
                size: m.size as u64,
                size_mb: (m.size as f64) / (1024.0 * 1024.0),
            }),
        });

        let table = inner.table.as_ref().map(|t| TableDebug {
            base: format!("0x{:X}", t.year_base),
            last_entry: format!("0x{:X}", t.year_base + (YEAR_COUNT - 1) * ENTRY_STRIDE),
            stride: ENTRY_STRIDE as u32,
            entry_count: YEAR_COUNT as u32,
            year_first: YEAR_FIRST,
            year_last: YEAR_LAST,
        });

        let mut hook_list: Vec<HookDebug> = Vec::new();
        let proc_ref = inner.proc.as_ref();
        let vcrt_base = proc_ref.and_then(|p| p.module("VCRUNTIME140.dll").ok().map(|m| m.base));
        if let Some(h) = inner.hooks.str_hook.as_ref() {
            hook_list.push(build_hook_debug(
                "Season suffix (YYYY/YY)",
                "VCRUNTIME140.dll",
                "7-byte memcpy leaf",
                vcrt_base,
                h.hook_addr,
                h.shellcode,
                h.shellcode_size,
                proc_ref,
            ));
        }
        if let Some(h) = inner.hooks.career_hook.as_ref() {
            hook_list.push(build_hook_debug(
                "Career range (YY-YY)",
                "VCRUNTIME140.dll",
                "5-byte memcpy leaf",
                vcrt_base,
                h.hook_addr,
                h.shellcode,
                h.shellcode_size,
                proc_ref,
            ));
        }
        if let Some(h) = inner.hooks.dword_hook.as_ref() {
            hook_list.push(build_hook_debug(
                "Year DWORD restoration",
                "VCRUNTIME140.dll",
                "DWORD memcpy leaf",
                vcrt_base,
                h.hook_addr,
                h.shellcode,
                h.shellcode_size,
                proc_ref,
            ));
        }

        let patch = inner.year_patcher.shift_applied.map(|shift| PatchDebug {
            shift,
            saved_entries: inner.year_patcher.saved_count(),
        });

        DebugInfo {
            host_os: std::env::consts::OS.into(),
            host_arch: std::env::consts::ARCH.into(),
            patcher_version: env!("CARGO_PKG_VERSION").into(),
            process,
            table,
            patch,
            hooks: hook_list,
            last_message: inner.last_message.clone(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct DebugInfo {
    pub host_os: String,
    pub host_arch: String,
    pub patcher_version: String,
    pub process: Option<ProcessDebug>,
    pub table: Option<TableDebug>,
    pub patch: Option<PatchDebug>,
    pub hooks: Vec<HookDebug>,
    pub last_message: String,
}

#[derive(Serialize, Clone)]
pub struct ProcessDebug {
    pub pid: u32,
    pub path: Option<String>,
    pub fm_module: Option<ModuleDebug>,
    pub vcrt_module: Option<ModuleDebug>,
}

#[derive(Serialize, Clone)]
pub struct ModuleDebug {
    pub base: String,
    pub size: u64,
    pub size_mb: f64,
}

#[derive(Serialize, Clone)]
pub struct TableDebug {
    pub base: String,
    pub last_entry: String,
    pub stride: u32,
    pub entry_count: u32,
    pub year_first: i32,
    pub year_last: i32,
}

#[derive(Serialize, Clone)]
pub struct PatchDebug {
    pub shift: i32,
    pub saved_entries: usize,
}

#[derive(Serialize, Clone)]
pub struct HookDebug {
    pub kind: String,
    pub module: String,
    pub leaf: String,
    pub hook_addr: String,
    pub hook_rva: Option<String>,
    pub shellcode_addr: String,
    pub shellcode_size: u64,
    pub jump_distance: i64,
    pub current_bytes: Option<String>,
    pub is_active: bool,
}

fn build_hook_debug(
    kind: &str,
    module: &str,
    leaf: &str,
    module_base: Option<usize>,
    hook_addr: usize,
    shellcode_addr: usize,
    shellcode_size: usize,
    proc: Option<&Arc<FmProcess>>,
) -> HookDebug {
    let current = proc.and_then(|p| p.read(hook_addr, 5).ok());
    let current_bytes = current
        .as_ref()
        .map(|b| b.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "));
    // Hook is active if first byte is JMP rel32 opcode (0xE9).
    let is_active = current.as_deref().map(|b| b.first() == Some(&0xE9)).unwrap_or(false);
    let jump_distance = shellcode_addr as i64 - (hook_addr as i64 + 5);
    let hook_rva = module_base.map(|b| format!("0x{:X}", hook_addr.wrapping_sub(b)));

    HookDebug {
        kind: kind.into(),
        module: module.into(),
        leaf: leaf.into(),
        hook_addr: format!("0x{:X}", hook_addr),
        hook_rva,
        shellcode_addr: format!("0x{:X}", shellcode_addr),
        shellcode_size: shellcode_size as u64,
        jump_distance,
        current_bytes,
        is_active,
    }
}
