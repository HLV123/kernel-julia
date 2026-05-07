// ============================================================
// symbols.rs -- Bảng ký hiệu (Stage 2: type-aware)
// ============================================================

use alloc::string::String;

pub const MAX_VARS: usize = 256;
pub const MAX_FUNCS: usize = 64;
pub const MAX_PARAMS: usize = 8;

#[derive(Clone)]
pub struct VarTable {
    names: [Option<String>; MAX_VARS],
    count: usize,
}

impl VarTable {
    pub fn new() -> Self {
        VarTable { names: core::array::from_fn(|_| None), count: 0 }
    }

    pub fn find(&self, name: &str) -> Option<usize> {
        for i in 0..self.count {
            if let Some(ref n) = self.names[i] {
                if n == name { return Some(i); }
            }
        }
        None
    }

    pub fn add(&mut self, name: &str) -> Option<usize> {
        if self.count >= MAX_VARS { return None; }
        let slot = self.count;
        self.names[slot] = Some(String::from(name));
        self.count += 1;
        Some(slot)
    }

    pub fn find_or_add(&mut self, name: &str) -> Option<usize> {
        if let Some(slot) = self.find(name) { return Some(slot); }
        self.add(name)
    }

    pub fn count(&self) -> usize { self.count }

    pub fn name_at(&self, slot: usize) -> Option<&str> {
        self.names.get(slot).and_then(|n| n.as_deref())
    }

    pub fn reset(&mut self) {
        for i in 0..self.count { self.names[i] = None; }
        self.count = 0;
    }
}

#[derive(Clone)]
pub struct FuncEntry {
    pub name: String,
    pub code_addr: usize,
    pub param_count: usize,
    pub param_slots: [usize; MAX_PARAMS],
}

#[derive(Clone)]
pub struct FuncTable {
    funcs: [Option<FuncEntry>; MAX_FUNCS],
    count: usize,
}

impl FuncTable {
    pub fn new() -> Self {
        FuncTable { funcs: core::array::from_fn(|_| None), count: 0 }
    }

    pub fn add(&mut self, name: &str, code_addr: usize, param_count: usize, param_slots: [usize; MAX_PARAMS]) -> bool {
        // Nếu hàm đã tồn tại → cập nhật (cho REPL)
        for i in 0..self.count {
            if let Some(ref mut f) = self.funcs[i] {
                if f.name == name {
                    f.code_addr = code_addr;
                    f.param_count = param_count;
                    f.param_slots = param_slots;
                    return true;
                }
            }
        }
        if self.count >= MAX_FUNCS { return false; }
        self.funcs[self.count] = Some(FuncEntry {
            name: String::from(name), code_addr, param_count, param_slots,
        });
        self.count += 1;
        true
    }

    pub fn find(&self, name: &str) -> Option<(usize, usize)> {
        for i in 0..self.count {
            if let Some(ref f) = self.funcs[i] {
                if f.name == name { return Some((f.code_addr, f.param_count)); }
            }
        }
        None
    }

    pub fn count(&self) -> usize { self.count }

    pub fn name_at(&self, idx: usize) -> Option<&str> {
        self.funcs.get(idx).and_then(|f| f.as_ref()).map(|f| f.name.as_str())
    }

    pub fn reset(&mut self) {
        for i in 0..self.count { self.funcs[i] = None; }
        self.count = 0;
    }
}
