use super::process::{FmProcess, Result};

pub const ENTRY_STRIDE: usize = 0x20;
pub const YEAR_FIRST: i32 = 1950;
pub const YEAR_LAST: i32 = 2050;
pub const YEAR_COUNT: usize = (YEAR_LAST - YEAR_FIRST + 1) as usize;

pub const ANCHOR_AOB: &[u8] = &[
    0x31, 0x39, 0x35, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0D, 0x00, 0x00, 0x00,
];

pub struct StringTable {
    pub year_base: usize,
}

impl StringTable {
    pub fn locate(proc: &FmProcess) -> Result<Self> {
        match proc.scan(ANCHOR_AOB) {
            Some(addr) => Ok(Self { year_base: addr }),
            None => Err(super::process::ProcError::Io(
                "year table anchor not found".into(),
            )),
        }
    }

    #[allow(dead_code)]
    pub fn year_addr(&self, year: i32) -> usize {
        self.year_base + ((year - YEAR_FIRST) as usize) * ENTRY_STRIDE
    }
}
