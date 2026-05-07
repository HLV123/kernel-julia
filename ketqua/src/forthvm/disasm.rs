// ============================================================
// disasm.rs -- Trình dịch ngược (Disassembler)
// Chuyển bytecode thành văn bản dễ đọc
// (chuyển từ 09-disasm.fs sang Rust)
// ============================================================

use alloc::string::String;
use alloc::format;
use crate::forthvm::opcode::*;
use crate::forthvm::memory::VmMemory;

/// Dịch ngược 1 lệnh tại addr
/// Trả về (text, next_addr)
pub fn disasm_one(memory: &VmMemory, addr: usize) -> (String, usize) {
    match memory.prog_read(addr) {
        Ok(cell) => {
            let opcode = unpack_opcode(cell);
            let arg = unpack_arg(cell);
            let name = opcode_name(opcode);

            let text = if opcode_has_arg(opcode) {
                format!("{:>3}: {:<12} {}", addr, name, arg)
            } else {
                format!("{:>3}: {}", addr, name)
            };
            (text, addr + 1)
        }
        Err(_) => {
            (format!("{:>3}: ???", addr), addr + 1)
        }
    }
}

/// Dịch ngược từ địa chỉ 0 đến end_addr
/// Trả về chuỗi text đầy đủ
pub fn disasm(memory: &VmMemory, end_addr: usize) -> String {
    let mut result = String::new();
    let mut addr = 0;
    while addr < end_addr {
        let (text, next) = disasm_one(memory, addr);
        result.push_str(&text);
        result.push('\n');
        addr = next;
    }
    result
}

/// Dịch ngược và in ra stdout qua bridge
pub fn disasm_print(vm: &crate::forthvm::vm::ForthVm, end_addr: usize) {
    let text = disasm(&vm.memory, end_addr);
    crate::print!("{}", text);
}
