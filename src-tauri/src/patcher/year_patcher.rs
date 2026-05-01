use super::process::{FmProcess, Result};
use super::string_table::{StringTable, ENTRY_STRIDE, YEAR_COUNT};

#[derive(Default)]
pub struct YearPatcher {
    pub shift_applied: Option<i32>,
    saved: Vec<(usize, String)>,
}

impl YearPatcher {
    pub fn new() -> Self {
        Self {
            shift_applied: None,
            saved: Vec::new(),
        }
    }

    pub fn patch(
        &mut self,
        proc: &FmProcess,
        table: &StringTable,
        shift: i32,
        min_year: i32,
    ) -> Result<usize> {
        if self.shift_applied.is_some() {
            return Err(super::process::ProcError::Io("already patched".into()));
        }
        self.saved.clear();
        let mut ok = 0usize;
        for i in 0..YEAR_COUNT {
            let addr = table.year_base + i * ENTRY_STRIDE;
            let s = match proc.read_ascii_digits(addr, 8) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if s.len() != 4 {
                continue;
            }
            let yr: i32 = match s.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if min_year > 0 && yr <= min_year {
                continue;
            }
            self.saved.push((addr, s));
            let new_val = (yr + shift).to_string();
            if proc.write_ascii_num(addr, &new_val).is_ok() {
                ok += 1;
            }
        }
        self.shift_applied = Some(shift);
        Ok(ok)
    }

    pub fn restore(&mut self, proc: &FmProcess) -> usize {
        if self.shift_applied.is_none() {
            return 0;
        }
        let mut ok = 0usize;
        for (addr, s) in self.saved.drain(..) {
            if proc.write_ascii_num(addr, &s).is_ok() {
                ok += 1;
            }
        }
        self.shift_applied = None;
        ok
    }

    pub fn saved_count(&self) -> usize {
        self.saved.len()
    }
}
