// ============================================================
// syscall_bridge.rs -- Cầu nối VM → Kernel (Stage 2)
// ============================================================

use alloc::string::String;
use crate::forthvm::value::{Value, StringPool, format_value, ArrayPool};

pub struct SyscallBridge;

impl SyscallBridge {
    pub fn new() -> Self { SyscallBridge }

    /// In giá trị + xuống dòng
    pub fn print_value(&self, val: &Value, strings: &StringPool, arrays: &ArrayPool) {
        let s = format_value(val, strings, arrays);
        crate::println!("{}", s);
    }

    /// In giá trị KHÔNG xuống dòng
    pub fn print_value_nolf(&self, val: &Value, strings: &StringPool, arrays: &ArrayPool) {
        let s = format_value(val, strings, arrays);
        crate::print!("{}", s);
    }
}
