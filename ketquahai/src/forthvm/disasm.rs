// ============================================================
// disasm.rs -- Trình dịch ngược (Stage 2)
// ============================================================

use alloc::string::String;
use alloc::format;
use crate::forthvm::opcode::*;
use crate::forthvm::memory::VmMemory;

pub fn disasm_one(memory: &VmMemory, addr: usize) -> (String, usize) {
    match memory.prog_read(addr) {
        Ok(cell) => {
            let opcode = unpack_opcode(cell);
            let arg = unpack_arg(cell);
            let name = opcode_name(opcode);
            let text = match opcode {
                OP_PUSH_INT | OP_PUSH_STR | OP_JMP | OP_JZ | OP_JNZ | OP_CALL |
                OP_LOAD | OP_STORE | OP_PUSH_R | OP_POP_R | OP_BUILTIN |
                OP_ARR_LITERAL => format!("{:>4}: {:<10} {}", addr, name, arg),
                _ => format!("{:>4}: {}", addr, name),
            };
            (text, addr + 1)
        }
        Err(_) => (format!("{:>4}: ???", addr), addr + 1),
    }
}

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
