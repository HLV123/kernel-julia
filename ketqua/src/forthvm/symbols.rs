// ============================================================
// symbols.rs -- Bảng ký hiệu
// Quản lý tên biến → slot dữ liệu, tên hàm → địa chỉ code
// (chuyển từ 13-symbols.fs sang Rust)
// ============================================================

use alloc::string::String;

// === Bảng biến (tối đa 64 biến) ===

pub const MAX_VARS: usize = 64;

/// Biến — tên + slot trong vùng Data
pub struct Variable {
    pub name: String,
    pub slot: usize,
}

/// Bảng biến — quản lý mapping tên biến → slot dữ liệu
pub struct VarTable {
    vars: [Option<String>; MAX_VARS],
    count: usize,
}

// Cần tạo mảng Option<String> mà không có Default cho array
impl VarTable {
    pub fn new() -> Self {
        VarTable {
            vars: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    /// Tìm biến theo tên → trả slot nếu thấy
    pub fn find(&self, name: &str) -> Option<usize> {
        for i in 0..self.count {
            if let Some(ref n) = self.vars[i] {
                if n == name {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Thêm biến mới → trả slot vừa tạo
    pub fn add(&mut self, name: &str) -> Option<usize> {
        if self.count >= MAX_VARS {
            return None;
        }
        let slot = self.count;
        self.vars[slot] = Some(String::from(name));
        self.count += 1;
        Some(slot)
    }

    /// Tìm hoặc tạo mới nếu chưa có
    pub fn find_or_add(&mut self, name: &str) -> Option<usize> {
        if let Some(slot) = self.find(name) {
            return Some(slot);
        }
        self.add(name)
    }

    /// Reset bảng biến
    pub fn reset(&mut self) {
        for i in 0..self.count {
            self.vars[i] = None;
        }
        self.count = 0;
    }
}

// === Bảng hàm (tối đa 16 hàm) ===

pub const MAX_FUNCS: usize = 16;

/// Hàm — tên + địa chỉ bytecode + số tham số
pub struct Function {
    pub name: String,
    pub code_addr: usize,
    pub param_count: usize,
}

/// Bảng hàm — quản lý mapping tên hàm → địa chỉ code
pub struct FuncTable {
    funcs: [Option<Function>; MAX_FUNCS],
    count: usize,
}

impl FuncTable {
    pub fn new() -> Self {
        FuncTable {
            funcs: core::array::from_fn(|_| None),
            count: 0,
        }
    }

    /// Đăng ký hàm mới
    pub fn add(&mut self, name: &str, code_addr: usize, param_count: usize) -> bool {
        if self.count >= MAX_FUNCS {
            return false;
        }
        self.funcs[self.count] = Some(Function {
            name: String::from(name),
            code_addr,
            param_count,
        });
        self.count += 1;
        true
    }

    /// Tìm hàm → trả (code_addr, param_count) nếu tìm thấy
    pub fn find(&self, name: &str) -> Option<(usize, usize)> {
        for i in 0..self.count {
            if let Some(ref f) = self.funcs[i] {
                if f.name == name {
                    return Some((f.code_addr, f.param_count));
                }
            }
        }
        None
    }

    /// Reset bảng hàm
    pub fn reset(&mut self) {
        for i in 0..self.count {
            self.funcs[i] = None;
        }
        self.count = 0;
    }
}
