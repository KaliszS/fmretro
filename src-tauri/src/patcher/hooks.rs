//! VCRUNTIME140 memcpy leaf hooks.
//! Direct ports of the Python shellcode builders (vcrt_hook.py,
//! vcrt_career_hook.py, vcrt_dword_hook.py).

use super::process::{FmProcess, Result};

// ── shellcode helpers ────────────────────────────────────────────────────────

struct Sc {
    bytes: Vec<u8>,
}

impl Sc {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }
    fn push(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }
    fn len(&self) -> usize {
        self.bytes.len()
    }
    /// Emit 6-byte near conditional jump (0F op rel32). Returns its offset.
    fn jcc32(&mut self, op2: u8) -> usize {
        let off = self.bytes.len();
        self.bytes.extend_from_slice(&[0x0F, op2, 0, 0, 0, 0]);
        off
    }
    /// Patch rel32 at jmp_off+2 so the jump lands at the current end of buffer.
    fn fix(&mut self, jmp_off: usize) {
        let disp = (self.bytes.len() as i64 - (jmp_off as i64 + 6)) as i32;
        self.bytes[jmp_off + 2..jmp_off + 6].copy_from_slice(&disp.to_le_bytes());
    }
    fn pack_q_at(&mut self, off: usize, value: u64) {
        self.bytes[off..off + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

fn jmp5(rel: i32) -> [u8; 5] {
    let b = rel.to_le_bytes();
    [0xE9, b[0], b[1], b[2], b[3]]
}

fn rel_jmp_check(from_after: usize, target: usize) -> Option<i32> {
    let rel = target as i64 - from_after as i64;
    if (-0x8000_0000..=0x7FFF_FFFF).contains(&rel) {
        Some(rel as i32)
    } else {
        None
    }
}

fn find_pattern_in_module(proc: &FmProcess, module: &str, pattern: &[u8]) -> Option<usize> {
    let info = proc.module(module).ok()?;
    const CHUNK: usize = 4 * 1024 * 1024;
    let mut off = 0usize;
    while off < info.size {
        let n = CHUNK.min(info.size - off);
        if let Ok(data) = proc.read(info.base + off, n) {
            if let Some(idx) = data.windows(pattern.len()).position(|w| w == pattern) {
                return Some(info.base + off + idx);
            }
        }
        off += n;
    }
    None
}

fn build_suffix_table(shift: i32) -> Vec<u8> {
    let s = shift.rem_euclid(100);
    let mut t = vec![0u8; 200];
    for i in 0..100 {
        let retro = (i - s).rem_euclid(100);
        t[(i * 2) as usize] = 0x30 + (retro / 10) as u8;
        t[(i * 2 + 1) as usize] = 0x30 + (retro % 10) as u8;
    }
    t
}

fn build_rev_table(shift: i32, min_year: i32) -> Vec<u8> {
    let mut t = vec![0u8; 400];
    for original in 1950..=min_year {
        let shifted_last2 = (original - shift).rem_euclid(100) as usize;
        let bytes = format!("{:04}", original).into_bytes();
        let dword = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let off = shifted_last2 * 4;
        t[off..off + 4].copy_from_slice(&dword.to_le_bytes());
    }
    t
}

// ── Hook 1: VCRTStrHook (7-byte leaf, season suffix YYYY/YY) ────────────────

const STR_SCAN: &[u8] = &[
    0x8B, 0x0A, 0x44, 0x0F, 0xB7, 0x42, 0x04, 0x44, 0x0F, 0xB6, 0x4A, 0x06, 0x89, 0x08, 0x66, 0x44,
    0x89, 0x40, 0x04, 0x44, 0x88, 0x48, 0x06, 0xC3,
];
const STR_HOOK_OFFSET: usize = 19;
const STR_ORIG: [u8; 5] = [0x44, 0x88, 0x48, 0x06, 0xC3];

pub struct VcrtStrHook {
    pub hook_addr: usize,
    pub shellcode: usize,
    pub shellcode_size: usize,
}

impl VcrtStrHook {
    pub fn install(
        proc: &FmProcess,
        shift: i32,
        min_year: i32,
        year_table_base: usize,
    ) -> Result<Self> {
        let site = find_pattern_in_module(proc, "vcruntime140.dll", STR_SCAN)
            .ok_or_else(|| super::process::ProcError::Io("VCRT 7-byte leaf not found".into()))?
            + STR_HOOK_OFFSET;
        let cur = proc.read(site, 5)?;
        if cur != STR_ORIG {
            return Err(super::process::ProcError::Io(format!(
                "unexpected bytes at hook site: {:02X?}",
                cur
            )));
        }
        let sc_addr = proc
            .alloc_near(site, 0x1000)
            .ok_or_else(|| super::process::ProcError::Io("alloc_near failed".into()))?;
        let sc = build_str_shellcode(sc_addr, shift, min_year, year_table_base);
        let sc_size = sc.len();
        if let Err(e) = proc.write(sc_addr, &sc) {
            proc.free(sc_addr);
            return Err(e);
        }
        let rel = match rel_jmp_check(site + 5, sc_addr) {
            Some(r) => r,
            None => {
                proc.free(sc_addr);
                return Err(super::process::ProcError::Io("rel32 out of range".into()));
            }
        };
        if let Err(e) = proc.vwrite(site, &jmp5(rel)) {
            proc.free(sc_addr);
            return Err(e);
        }
        Ok(Self {
            hook_addr: site,
            shellcode: sc_addr,
            shellcode_size: sc_size,
        })
    }

    pub fn remove(self, proc: &FmProcess) {
        let _ = proc.vwrite(self.hook_addr, &STR_ORIG);
        proc.free(self.shellcode);
    }
}

fn build_str_shellcode(sc_addr: usize, shift: i32, min_year: i32, year_table_base: usize) -> Vec<u8> {
    let shift_const = (shift.rem_euclid(100) + 1).rem_euclid(100) as u8;
    let do_yyyy = min_year > 0;
    let threshold_be: u32 = if min_year > 0 {
        let threshold = min_year - shift;
        let s = format!("{:04}", threshold);
        u32::from_be_bytes([s.as_bytes()[0], s.as_bytes()[1], s.as_bytes()[2], s.as_bytes()[3]])
    } else {
        0
    };

    let mut c = Sc::new();
    c.push(&[0x41, 0x52]); // push r10
    c.push(&[0x41, 0x53]); // push r11
    c.push(&[0x41, 0x54]); // push r12

    // [rax+4] == '/'
    c.push(&[0x80, 0x78, 0x04, 0x2F]);
    let j1 = c.jcc32(0x85);

    // suffix digit 1
    c.push(&[0x44, 0x0F, 0xB6, 0x50, 0x05]);
    c.push(&[0x41, 0x83, 0xEA, 0x30]);
    c.push(&[0x41, 0x83, 0xFA, 0x09]);
    let j2 = c.jcc32(0x87);

    // suffix digit 2
    c.push(&[0x45, 0x0F, 0xB6, 0xD9]);
    c.push(&[0x41, 0x83, 0xEB, 0x30]);
    c.push(&[0x41, 0x83, 0xFB, 0x09]);
    let j3 = c.jcc32(0x87);

    c.push(&[0x45, 0x6B, 0xD2, 0x0A]);
    c.push(&[0x45, 0x03, 0xD3]);

    // year_last2 from [rax+2..3]
    c.push(&[0x44, 0x0F, 0xB6, 0x58, 0x02]);
    c.push(&[0x41, 0x83, 0xEB, 0x30]);
    c.push(&[0x45, 0x6B, 0xDB, 0x0A]);
    c.push(&[0x44, 0x0F, 0xB6, 0x60, 0x03]);
    c.push(&[0x41, 0x83, 0xEC, 0x30]);
    c.push(&[0x45, 0x03, 0xDC]);

    let mut j_miny: Option<usize> = None;
    if min_year > 0 {
        c.push(&[0x44, 0x8B, 0x20]);
        c.push(&[0x41, 0x0F, 0xCC]);
        c.push(&[0x41, 0x81, 0xFC]);
        c.push(&threshold_be.to_le_bytes());
        j_miny = Some(c.jcc32(0x86));
    }

    // suffix transform
    c.push(&[0x41, 0x83, 0xC3, shift_const]);
    c.push(&[0x41, 0x83, 0xFB, 0x63]);
    let j_wrap = c.jcc32(0x86);
    c.push(&[0x41, 0x83, 0xEB, 0x64]);
    c.fix(j_wrap);

    c.push(&[0x45, 0x3B, 0xD3]);
    let j4 = c.jcc32(0x85);

    let mov_r11_off = c.len();
    c.push(&[0x49, 0xBB, 0, 0, 0, 0, 0, 0, 0, 0]);
    c.push(&[0x47, 0x0F, 0xB7, 0x14, 0x53]);
    c.push(&[0x44, 0x88, 0x50, 0x05]);
    c.push(&[0x41, 0xC1, 0xEA, 0x08]);
    c.push(&[0x44, 0x88, 0x50, 0x06]);
    c.push(&[0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A, 0xC3]);

    // .fix_year
    let mut j_genuine: Option<usize> = None;
    let mut j_rev_zero: Option<usize> = None;
    let mut rev_table_ptr_off: Option<usize> = None;
    if min_year > 0 {
        c.fix(j_miny.unwrap());
        c.push(&[0x45, 0x8B, 0xE3]);
        c.push(&[0x41, 0x83, 0xC4, shift_const]);
        c.push(&[0x41, 0x83, 0xFC, 0x63]);
        let j_fyr_wrap = c.jcc32(0x86);
        c.push(&[0x41, 0x83, 0xEC, 0x64]);
        c.fix(j_fyr_wrap);
        c.push(&[0x45, 0x3B, 0xD4]);
        j_genuine = Some(c.jcc32(0x85));
        rev_table_ptr_off = Some(c.len());
        c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.push(&[0x47, 0x8B, 0x14, 0x9A]);
        c.push(&[0x45, 0x85, 0xD2]);
        j_rev_zero = Some(c.jcc32(0x84));
        c.push(&[0x44, 0x89, 0x10]);
    }

    // .maybe_yyyy (path B)
    let mut j_yyyy_rdx_lo: Option<usize> = None;
    let mut j_yyyy_rdx_hi: Option<usize> = None;
    let mut j_yyyy_no19: Option<usize> = None;
    let mut j_yyyy_oob: Option<usize> = None;
    let mut j_yyyy_zero: Option<usize> = None;
    let mut year_ptr_off: Option<usize> = None;
    let mut year_base_ptr_off: Option<usize> = None;

    if do_yyyy {
        c.fix(j1);
        year_base_ptr_off = Some(c.len());
        c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.push(&[0x49, 0x3B, 0xD2]);
        j_yyyy_rdx_lo = Some(c.jcc32(0x82));
        c.push(&[0x49, 0x81, 0xC2]);
        c.push(&(101u32 * 32).to_le_bytes());
        c.push(&[0x49, 0x3B, 0xD2]);
        j_yyyy_rdx_hi = Some(c.jcc32(0x83));

        c.push(&[0x44, 0x8B, 0x10]);
        c.push(&[0x45, 0x8B, 0xDA]);
        c.push(&[0x41, 0x81, 0xE3, 0xFF, 0xFF, 0x00, 0x00]);
        c.push(&[0x41, 0x81, 0xFB, 0x31, 0x39, 0x00, 0x00]);
        j_yyyy_no19 = Some(c.jcc32(0x85));

        c.push(&[0x45, 0x8B, 0xDA]);
        c.push(&[0x41, 0xC1, 0xEB, 0x10]);
        c.push(&[0x45, 0x0F, 0xB6, 0xE3]);
        c.push(&[0x41, 0x83, 0xEC, 0x30]);
        c.push(&[0x45, 0x6B, 0xE4, 0x0A]);
        c.push(&[0x41, 0xC1, 0xEB, 0x08]);
        c.push(&[0x45, 0x0F, 0xB6, 0xDB]);
        c.push(&[0x41, 0x83, 0xEB, 0x30]);
        c.push(&[0x45, 0x03, 0xDC]);

        c.push(&[0x41, 0x83, 0xFB, 0x63]);
        j_yyyy_oob = Some(c.jcc32(0x87));

        year_ptr_off = Some(c.len());
        c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.push(&[0x47, 0x8B, 0x14, 0x9A]);
        c.push(&[0x45, 0x85, 0xD2]);
        j_yyyy_zero = Some(c.jcc32(0x84));
        c.push(&[0x44, 0x89, 0x10]);
    }

    // .pass
    if !do_yyyy {
        c.fix(j1);
    }
    let mut all: Vec<usize> = vec![j2, j3, j4];
    for opt in [j_genuine, j_rev_zero, j_yyyy_rdx_lo, j_yyyy_rdx_hi, j_yyyy_no19, j_yyyy_oob, j_yyyy_zero] {
        if let Some(j) = opt {
            all.push(j);
        }
    }
    for j in all {
        c.fix(j);
    }

    c.push(&[0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A]);
    c.push(&[0x44, 0x88, 0x48, 0x06]); // original
    c.push(&[0xC3]);

    // tables
    let suffix_table_addr = sc_addr + c.len();
    c.pack_q_at(mov_r11_off + 2, suffix_table_addr as u64);
    let suffix_bytes = build_suffix_table(shift);

    let rev_bytes: Vec<u8> = if min_year > 0 {
        if let Some(off) = rev_table_ptr_off {
            let rev_addr = sc_addr + c.len() + suffix_bytes.len();
            c.pack_q_at(off + 2, rev_addr as u64);
            build_rev_table(shift, min_year)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let entry_rev_bytes: Vec<u8> = if do_yyyy {
        if let Some(off) = year_ptr_off {
            let entry_rev_addr = sc_addr + c.len() + suffix_bytes.len() + rev_bytes.len();
            c.pack_q_at(off + 2, entry_rev_addr as u64);
            build_rev_table(shift, min_year)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if do_yyyy {
        if let Some(off) = year_base_ptr_off {
            if year_table_base != 0 {
                c.pack_q_at(off + 2, year_table_base as u64);
            }
        }
    }

    let mut out = c.into_bytes();
    out.extend_from_slice(&suffix_bytes);
    out.extend_from_slice(&rev_bytes);
    out.extend_from_slice(&entry_rev_bytes);
    out
}

// ── Hook 2: VCRTCareerHook (5-byte leaf, "YY-YY") ───────────────────────────

const CAR_SCAN: &[u8] = &[
    0x8B, 0x0A, 0x44, 0x0F, 0xB6, 0x42, 0x04, 0x89, 0x08, 0x44, 0x88, 0x40, 0x04, 0xC3,
];
const CAR_HOOK_OFFSET: usize = 9;
const CAR_ORIG: [u8; 5] = [0x44, 0x88, 0x40, 0x04, 0xC3];

pub struct VcrtCareerHook {
    pub hook_addr: usize,
    pub shellcode: usize,
    pub shellcode_size: usize,
}

impl VcrtCareerHook {
    pub fn install(
        proc: &FmProcess,
        shift: i32,
        min_year: i32,
        _year_table_base: usize,
    ) -> Result<Self> {
        let site = find_pattern_in_module(proc, "vcruntime140.dll", CAR_SCAN)
            .ok_or_else(|| super::process::ProcError::Io("VCRT 5-byte leaf not found".into()))?
            + CAR_HOOK_OFFSET;
        let cur = proc.read(site, 5)?;
        if cur != CAR_ORIG {
            return Err(super::process::ProcError::Io(format!(
                "unexpected bytes at career hook: {:02X?}",
                cur
            )));
        }
        let sc_addr = proc
            .alloc_near(site, 0x1000)
            .ok_or_else(|| super::process::ProcError::Io("alloc_near failed".into()))?;
        let sc = build_career_shellcode(sc_addr, shift, min_year);
        let sc_size = sc.len();
        if let Err(e) = proc.write(sc_addr, &sc) {
            proc.free(sc_addr);
            return Err(e);
        }
        let rel = match rel_jmp_check(site + 5, sc_addr) {
            Some(r) => r,
            None => {
                proc.free(sc_addr);
                return Err(super::process::ProcError::Io("rel32 out of range".into()));
            }
        };
        if let Err(e) = proc.vwrite(site, &jmp5(rel)) {
            proc.free(sc_addr);
            return Err(e);
        }
        Ok(Self {
            hook_addr: site,
            shellcode: sc_addr,
            shellcode_size: sc_size,
        })
    }

    pub fn remove(self, proc: &FmProcess) {
        let _ = proc.vwrite(self.hook_addr, &CAR_ORIG);
        proc.free(self.shellcode);
    }
}

fn build_career_shellcode(sc_addr: usize, shift: i32, min_year: i32) -> Vec<u8> {
    let mut c = Sc::new();
    c.push(&[0x41, 0x52, 0x41, 0x53, 0x41, 0x54]);

    // [rax+2] == '-'
    c.push(&[0x80, 0x78, 0x02, 0x2D]);
    let j1 = c.jcc32(0x85);

    c.push(&[0x44, 0x0F, 0xB6, 0x10]);
    c.push(&[0x41, 0x83, 0xEA, 0x30]);
    c.push(&[0x41, 0x83, 0xFA, 0x09]);
    let j2 = c.jcc32(0x83);

    c.push(&[0x44, 0x0F, 0xB6, 0x58, 0x01]);
    c.push(&[0x41, 0x83, 0xEB, 0x30]);
    c.push(&[0x41, 0x83, 0xFB, 0x09]);
    let j3 = c.jcc32(0x83);

    c.push(&[0x45, 0x6B, 0xD2, 0x0A]);
    c.push(&[0x45, 0x03, 0xD3]);

    c.push(&[0x41, 0x83, 0xFA, 0x32]);
    let j4 = c.jcc32(0x83);

    c.push(&[0x44, 0x0F, 0xB6, 0x58, 0x03]);
    c.push(&[0x41, 0x83, 0xEB, 0x30]);
    c.push(&[0x41, 0x83, 0xFB, 0x09]);
    let j5 = c.jcc32(0x83);

    c.push(&[0x45, 0x0F, 0xB6, 0xE0]);
    c.push(&[0x41, 0x83, 0xEC, 0x30]);
    c.push(&[0x41, 0x83, 0xFC, 0x09]);
    let j6 = c.jcc32(0x83);

    c.push(&[0x45, 0x6B, 0xDB, 0x0A]);
    c.push(&[0x45, 0x03, 0xDC]);

    let mov_r12_off = c.len();
    c.push(&[0x49, 0xBC, 0, 0, 0, 0, 0, 0, 0, 0]);

    c.push(&[0x47, 0x0F, 0xB7, 0x14, 0x54]);
    c.push(&[0x44, 0x88, 0x10]);
    c.push(&[0x41, 0xC1, 0xEA, 0x08]);
    c.push(&[0x44, 0x88, 0x50, 0x01]);

    c.push(&[0x47, 0x0F, 0xB7, 0x1C, 0x5C]);
    c.push(&[0x44, 0x88, 0x58, 0x03]);
    c.push(&[0x41, 0xC1, 0xEB, 0x08]);
    c.push(&[0x45, 0x88, 0xD8]);

    c.push(&[0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A]);
    c.push(&[0x44, 0x88, 0x40, 0x04]);
    c.push(&[0xC3]);

    // .maybe_yyyy
    let mut j_yyyy_no19: Option<usize> = None;
    let mut j_yyyy_oob: Option<usize> = None;
    let mut j_yyyy_zero: Option<usize> = None;
    let mut rev_table_ptr_off: Option<usize> = None;

    if min_year > 0 {
        c.fix(j1);
        c.push(&[0x44, 0x8B, 0x10]);
        c.push(&[0x45, 0x8B, 0xDA]);
        c.push(&[0x41, 0x81, 0xE3, 0xFF, 0xFF, 0x00, 0x00]);
        c.push(&[0x41, 0x81, 0xFB, 0x31, 0x39, 0x00, 0x00]);
        j_yyyy_no19 = Some(c.jcc32(0x85));

        c.push(&[0x45, 0x8B, 0xDA]);
        c.push(&[0x41, 0xC1, 0xEB, 0x10]);
        c.push(&[0x45, 0x0F, 0xB6, 0xE3]);
        c.push(&[0x41, 0x83, 0xEC, 0x30]);
        c.push(&[0x45, 0x6B, 0xE4, 0x0A]);
        c.push(&[0x41, 0xC1, 0xEB, 0x08]);
        c.push(&[0x45, 0x0F, 0xB6, 0xDB]);
        c.push(&[0x41, 0x83, 0xEB, 0x30]);
        c.push(&[0x45, 0x03, 0xDC]);

        c.push(&[0x41, 0x83, 0xFB, 0x63]);
        j_yyyy_oob = Some(c.jcc32(0x87));

        rev_table_ptr_off = Some(c.len());
        c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.push(&[0x47, 0x8B, 0x14, 0x9A]);
        c.push(&[0x45, 0x85, 0xD2]);
        j_yyyy_zero = Some(c.jcc32(0x84));
        c.push(&[0x44, 0x89, 0x10]);
    }

    // .pass
    if min_year == 0 {
        c.fix(j1);
    }
    for j in [j2, j3, j4, j5, j6] {
        c.fix(j);
    }
    if min_year > 0 {
        for j in [j_yyyy_no19, j_yyyy_oob, j_yyyy_zero].into_iter().flatten() {
            c.fix(j);
        }
    }

    c.push(&[0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A]);
    c.push(&[0x44, 0x88, 0x40, 0x04]);
    c.push(&[0xC3]);

    // tables
    let table_addr = sc_addr + c.len();
    c.pack_q_at(mov_r12_off + 2, table_addr as u64);
    let suffix_bytes = build_suffix_table(shift);

    let rev_bytes: Vec<u8> = if min_year > 0 {
        if let Some(off) = rev_table_ptr_off {
            let rev_addr = sc_addr + c.len() + suffix_bytes.len();
            c.pack_q_at(off + 2, rev_addr as u64);
            build_rev_table(shift, min_year)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let mut out = c.into_bytes();
    out.extend_from_slice(&suffix_bytes);
    out.extend_from_slice(&rev_bytes);
    out
}

// ── Hook 3: VCRTDWordHook (4-byte leaf, YYYY restoration) ───────────────────

const DW_SCAN: &[u8] = &[0x88, 0x08, 0xC3, 0x8B, 0x0A, 0x89, 0x08, 0xC3];
const DW_HOOK_OFFSET: usize = 3;
const DW_ORIG: [u8; 5] = [0x8B, 0x0A, 0x89, 0x08, 0xC3];

pub struct VcrtDWordHook {
    pub hook_addr: usize,
    pub shellcode: usize,
    pub shellcode_size: usize,
}

impl VcrtDWordHook {
    pub fn install(
        proc: &FmProcess,
        shift: i32,
        min_year: i32,
        year_table_base: usize,
    ) -> Result<Self> {
        if min_year <= 0 {
            return Err(super::process::ProcError::Io(
                "DWord hook requires min_year > 0".into(),
            ));
        }
        let site = find_pattern_in_module(proc, "vcruntime140.dll", DW_SCAN)
            .ok_or_else(|| super::process::ProcError::Io("VCRT DWORD leaf not found".into()))?
            + DW_HOOK_OFFSET;
        let cur = proc.read(site, 5)?;
        if cur != DW_ORIG {
            return Err(super::process::ProcError::Io(format!(
                "unexpected bytes at dword hook: {:02X?}",
                cur
            )));
        }
        let sc_addr = proc
            .alloc_near(site, 0x1000)
            .ok_or_else(|| super::process::ProcError::Io("alloc_near failed".into()))?;
        let sc = build_dword_shellcode(sc_addr, shift, min_year, year_table_base);
        let sc_size = sc.len();
        if let Err(e) = proc.write(sc_addr, &sc) {
            proc.free(sc_addr);
            return Err(e);
        }
        let rel = match rel_jmp_check(site + 5, sc_addr) {
            Some(r) => r,
            None => {
                proc.free(sc_addr);
                return Err(super::process::ProcError::Io("rel32 out of range".into()));
            }
        };
        if let Err(e) = proc.vwrite(site, &jmp5(rel)) {
            proc.free(sc_addr);
            return Err(e);
        }
        Ok(Self {
            hook_addr: site,
            shellcode: sc_addr,
            shellcode_size: sc_size,
        })
    }

    pub fn remove(self, proc: &FmProcess) {
        let _ = proc.vwrite(self.hook_addr, &DW_ORIG);
        proc.free(self.shellcode);
    }
}

fn build_dword_shellcode(sc_addr: usize, shift: i32, min_year: i32, year_table_base: usize) -> Vec<u8> {
    let mut c = Sc::new();
    c.push(&[0x41, 0x52, 0x41, 0x53, 0x41, 0x54]);
    c.push(&[0x8B, 0x0A]); // mov ecx, [rdx]

    let mut ytb_ptr_off: Option<usize> = None;
    let mut j_rdx_lo: Option<usize> = None;
    let mut j_rdx_hi: Option<usize> = None;
    if year_table_base != 0 {
        ytb_ptr_off = Some(c.len());
        c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.push(&[0x49, 0x3B, 0xD2]);
        j_rdx_lo = Some(c.jcc32(0x82));
        c.push(&[0x49, 0x81, 0xC2]);
        c.push(&(101u32 * 32).to_le_bytes());
        c.push(&[0x49, 0x3B, 0xD2]);
        j_rdx_hi = Some(c.jcc32(0x83));
    }

    c.push(&[0x44, 0x8B, 0xD9]);
    c.push(&[0x41, 0x81, 0xE3, 0xFF, 0xFF, 0x00, 0x00]);
    c.push(&[0x41, 0x81, 0xFB, 0x31, 0x39, 0x00, 0x00]);
    let j_no19 = c.jcc32(0x85);

    c.push(&[0x44, 0x8B, 0xD9]);
    c.push(&[0x41, 0xC1, 0xEB, 0x10]);
    c.push(&[0x45, 0x0F, 0xB6, 0xE3]);
    c.push(&[0x41, 0x83, 0xEC, 0x30]);
    c.push(&[0x45, 0x6B, 0xE4, 0x0A]);
    c.push(&[0x41, 0xC1, 0xEB, 0x08]);
    c.push(&[0x45, 0x0F, 0xB6, 0xDB]);
    c.push(&[0x41, 0x83, 0xEB, 0x30]);
    c.push(&[0x45, 0x03, 0xDC]);

    c.push(&[0x41, 0x83, 0xFB, 0x63]);
    let j_oob = c.jcc32(0x87);

    let rev_ptr_off = c.len();
    c.push(&[0x49, 0xBA, 0, 0, 0, 0, 0, 0, 0, 0]);
    c.push(&[0x47, 0x8B, 0x14, 0x9A]);
    c.push(&[0x45, 0x85, 0xD2]);
    let j_zero = c.jcc32(0x84);

    c.push(&[0x44, 0x89, 0xD1]);

    // .pass
    c.fix(j_no19);
    c.fix(j_oob);
    c.fix(j_zero);
    if let (Some(a), Some(b)) = (j_rdx_lo, j_rdx_hi) {
        c.fix(a);
        c.fix(b);
    }

    c.push(&[0x41, 0x5C, 0x41, 0x5B, 0x41, 0x5A]);
    c.push(&[0x89, 0x08]);
    c.push(&[0xC3]);

    let rev_addr = sc_addr + c.len();
    c.pack_q_at(rev_ptr_off + 2, rev_addr as u64);
    if let Some(off) = ytb_ptr_off {
        c.pack_q_at(off + 2, year_table_base as u64);
    }

    let mut out = c.into_bytes();
    out.extend_from_slice(&build_rev_table(shift, min_year));
    out
}
