// ============================================================
// assembler.rs -- Placeholder (Stage 2: ít dùng trực tiếp)
// ============================================================

use alloc::string::String;
use crate::forthvm::vm::ForthVm;

#[derive(Debug)]
pub enum AsmError {
    UnknownInstruction(String),
    ProgramTooLarge,
}

/// Assemble vẫn giữ cho tương thích, nhưng Julia syntax là chính
pub fn assemble_into(_vm: &mut ForthVm, _source: &str) -> Result<usize, AsmError> {
    // Stage 2 focuses on Julia syntax, not raw assembly
    Err(AsmError::UnknownInstruction(String::from("use julia syntax instead")))
}
