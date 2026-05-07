// ============================================================
// assembler.rs -- Trình hợp dịch văn bản (Phase 6)
// Chuyển chuỗi assembly "PUSH 5 ADD HALT" → bytecode
// (chuyển từ 11-assembler.fs sang Rust)
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::forthvm::opcode::*;
use crate::forthvm::vm::ForthVm;

/// Lỗi assembler
#[derive(Debug)]
pub enum AsmError {
    UnknownInstruction(String),
    MissingArgument(String),
    InvalidNumber(String),
    ProgramTooLarge,
}

/// Bộ phát bytecode — ghi lệnh vào VM memory
pub struct Assembler {
    /// Vị trí ghi bytecode tiếp theo
    pub emit_ptr: usize,
}

impl Assembler {
    pub fn new() -> Self {
        Assembler { emit_ptr: 0 }
    }

    /// Reset assembler
    pub fn reset(&mut self) {
        self.emit_ptr = 0;
    }

    /// Phát 1 cell bytecode vào VM
    pub fn emit(&mut self, vm: &mut ForthVm, cell: u32) -> Result<(), AsmError> {
        if self.emit_ptr >= PROG_SIZE {
            return Err(AsmError::ProgramTooLarge);
        }
        let _ = vm.memory.prog_write(self.emit_ptr, cell);
        self.emit_ptr += 1;
        Ok(())
    }

    /// Phát lệnh đã đóng gói (arg + opcode)
    pub fn emit_packed(&mut self, vm: &mut ForthVm, arg: u32, opcode: u8) -> Result<(), AsmError> {
        self.emit(vm, pack(arg, opcode))
    }

    /// Số lệnh đã phát
    pub fn count(&self) -> usize {
        self.emit_ptr
    }

    /// Biên dịch chuỗi assembly → bytecode trong VM
    /// Format: "PUSH 5 PUSH 6 ADD PRINT HALT"
    pub fn assemble(&mut self, vm: &mut ForthVm, source: &str) -> Result<(), AsmError> {
        let mut tokens = Tokenizer::new(source);

        while let Some(tok) = tokens.next_token() {
            match tok.to_uppercase().as_str() {
                // Lệnh có tham số
                "PUSH" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_PUSH)?;
                }
                "PUSH_R" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_PUSH_R)?;
                }
                "POP_R" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_POP_R)?;
                }
                "JMP" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_JMP)?;
                }
                "JZ" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_JZ)?;
                }
                "JGT" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_JGT)?;
                }
                "CALL" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_CALL)?;
                }
                "LOAD_DATA" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_LOAD_DATA)?;
                }
                "STORE_DATA" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_STORE_DATA)?;
                }
                "ALLOC" => {
                    let arg = tokens.expect_number(&tok)?;
                    self.emit_packed(vm, arg, OP_ALLOC)?;
                }
                "SAVE" => {
                    self.emit_packed(vm, 0, OP_SAVE)?;
                }
                "RESTORE" => {
                    self.emit_packed(vm, 0, OP_RESTORE)?;
                }
                // Lệnh không tham số
                "ADD"        => self.emit_packed(vm, 0, OP_ADD)?,
                "SUB"        => self.emit_packed(vm, 0, OP_SUB)?,
                "MUL"        => self.emit_packed(vm, 0, OP_MUL)?,
                "PRINT"      => self.emit_packed(vm, 0, OP_PRINT)?,
                "RET"        => self.emit_packed(vm, 0, OP_RET)?,
                "HALT"       => self.emit_packed(vm, 0, OP_HALT)?,
                "DUP"        => self.emit_packed(vm, 0, OP_DUP)?,
                "DROP"       => self.emit_packed(vm, 0, OP_DROP)?,
                "SWAP"       => self.emit_packed(vm, 0, OP_SWAP)?,
                "FREE"       => self.emit_packed(vm, 0, OP_FREE)?,
                "HEAP_LOAD"  => self.emit_packed(vm, 0, OP_HEAP_LOAD)?,
                "HEAP_STORE" => self.emit_packed(vm, 0, OP_HEAP_STORE)?,
                "CMP_EQ"     => self.emit_packed(vm, 0, OP_CMP_EQ)?,
                "CMP_GT"     => self.emit_packed(vm, 0, OP_CMP_GT)?,
                "FRAME_SAVE" => self.emit_packed(vm, 0, OP_FRAME_SAVE)?,
                other => return Err(AsmError::UnknownInstruction(String::from(other))),
            }
        }

        Ok(())
    }
}

/// Convenience: khởi tạo VM, assemble, và sẵn sàng chạy
pub fn assemble_into(vm: &mut ForthVm, source: &str) -> Result<usize, AsmError> {
    vm.reset();
    let mut asm = Assembler::new();
    asm.assemble(vm, source)?;
    vm.pc = 0;
    Ok(asm.count())
}

// --- Tokenizer đơn giản ---

struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a str) -> Self {
        Tokenizer { src, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        let bytes = self.src.as_bytes();
        while self.pos < bytes.len() && (bytes[self.pos] == b' ' || bytes[self.pos] == b'\t'
            || bytes[self.pos] == b'\n' || bytes[self.pos] == b'\r') {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Option<String> {
        self.skip_whitespace();
        if self.pos >= self.src.len() {
            return None;
        }
        let start = self.pos;
        let bytes = self.src.as_bytes();
        while self.pos < bytes.len() && bytes[self.pos] != b' ' && bytes[self.pos] != b'\t'
            && bytes[self.pos] != b'\n' && bytes[self.pos] != b'\r' {
            self.pos += 1;
        }
        Some(String::from(&self.src[start..self.pos]))
    }

    fn expect_number(&mut self, instr_name: &str) -> Result<u32, AsmError> {
        match self.next_token() {
            Some(tok) => {
                // Hỗ trợ số âm (sign-extend)
                if tok.starts_with('-') {
                    match tok.parse::<i32>() {
                        Ok(n) => Ok(n as u32),
                        Err(_) => Err(AsmError::InvalidNumber(tok)),
                    }
                } else {
                    match tok.parse::<u32>() {
                        Ok(n) => Ok(n),
                        Err(_) => Err(AsmError::InvalidNumber(tok)),
                    }
                }
            }
            None => Err(AsmError::MissingArgument(String::from(instr_name))),
        }
    }
}
