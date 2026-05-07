// ============================================================
// compiler/mod.rs — Trình biên dịch Julia (Stage 2)
// Struct chính, emit helpers, public API
// ============================================================

pub mod expr;
pub mod stmt;

use alloc::string::String;
use alloc::vec::Vec;
use crate::forthvm::opcode::*;
use crate::forthvm::lexer::{Lexer, TokenKind};
use crate::forthvm::symbols::{VarTable, FuncTable, MAX_PARAMS};
use crate::forthvm::vm::ForthVm;
use crate::forthvm::builtins;

#[derive(Debug)]
pub enum CompileError {
    UnexpectedToken(String),
    MissingParen,
    MissingEnd,
    MissingBracket,
    MissingFunctionName,
    MissingParamName,
    FunctionNotFound(String),
    TooManyVariables,
    TooManyFunctions,
    ProgramTooLarge,
    InvalidString,
}

/// Loop context cho break/continue
struct LoopCtx {
    break_patches: Vec<usize>,   // Vị trí JMP cần patch khi break
    continue_patches: Vec<usize>, // Vị trí JMP cần patch khi continue (dùng cho for loop)
    continue_target: Option<usize>, // Địa chỉ đầu vòng lặp cho continue (dùng cho while loop)
}

/// Compiler chính
pub struct Compiler<'a> {
    pub lexer: Lexer,
    pub vm: &'a mut ForthVm,
    pub vars: VarTable,
    pub funcs: FuncTable,
    pub emit_ptr: usize,
    saved_name: String,
    param_slots: [usize; MAX_PARAMS],
    param_count: usize,
    loop_stack: Vec<LoopCtx>,
    /// Cho REPL: giữ lại emit_ptr bắt đầu để có thể chạy incremental
    pub code_start: usize,
    /// Có phải statement expression (auto-print trong REPL)?
    pub is_expr_stmt: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(vm: &'a mut ForthVm, source: &str) -> Self {
        Compiler {
            lexer: Lexer::new(source),
            vm,
            vars: VarTable::new(),
            funcs: FuncTable::new(),
            emit_ptr: 0,
            saved_name: String::new(),
            param_slots: [0; MAX_PARAMS],
            param_count: 0,
            loop_stack: Vec::new(),
            code_start: 0,
            is_expr_stmt: false,
        }
    }

    /// Tạo compiler tiếp tục từ trạng thái cũ (cho REPL)
    pub fn new_repl(
        vm: &'a mut ForthVm, source: &str,
        vars: VarTable, funcs: FuncTable, emit_ptr: usize,
    ) -> Self {
        Compiler {
            lexer: Lexer::new(source),
            vm, vars, funcs,
            emit_ptr,
            saved_name: String::new(),
            param_slots: [0; MAX_PARAMS],
            param_count: 0,
            loop_stack: Vec::new(),
            code_start: emit_ptr,
            is_expr_stmt: false,
        }
    }

    // --- Emit helpers ---

    pub fn here(&self) -> usize { self.emit_ptr }

    pub fn emit(&mut self, arg: u32, opcode: u8) -> Result<(), CompileError> {
        if self.emit_ptr >= PROG_SIZE { return Err(CompileError::ProgramTooLarge); }
        let _ = self.vm.memory.prog_write(self.emit_ptr, pack(arg, opcode));
        self.emit_ptr += 1;
        Ok(())
    }

    pub fn emit_jz_placeholder(&mut self) -> Result<usize, CompileError> {
        let p = self.here();
        self.emit(0, OP_JZ)?;
        Ok(p)
    }

    pub fn emit_jmp_placeholder(&mut self) -> Result<usize, CompileError> {
        let p = self.here();
        self.emit(0, OP_JMP)?;
        Ok(p)
    }

    pub fn patch(&mut self, addr: usize, target: usize) {
        if let Ok(cell) = self.vm.memory.prog_read(addr) {
            let opcode = cell & 0xFF;
            let _ = self.vm.memory.prog_write(addr, (target as u32) << 8 | opcode);
        }
    }

    pub fn skip_newlines(&mut self) {
        while self.lexer.kind() == TokenKind::Newline { self.lexer.next(); }
    }

    pub fn save_name(&mut self) {
        self.saved_name = self.lexer.current.str_val.clone();
    }

    pub fn saved_name(&self) -> &str { &self.saved_name }

    // --- String interpolation helper ---

    /// Biên dịch chuỗi có $var hoặc $(expr) thành OP_PUSH_STR + OP_STR_CONCAT
    pub fn compile_interpolated_string(&mut self, raw: &str) -> Result<(), CompileError> {
        let bytes = raw.as_bytes();
        let mut i = 0;
        let mut first = true;

        while i < bytes.len() {
            if bytes[i] == b'$' && i + 1 < bytes.len() {
                i += 1;
                if bytes[i] == b'(' {
                    // $(expr) — tạo sub-lexer/compiler phức tạp, đơn giản hóa: chỉ hỗ trợ $var
                    // Tìm closing )
                    i += 1;
                    let start = i;
                    let mut depth = 1;
                    while i < bytes.len() && depth > 0 {
                        if bytes[i] == b'(' { depth += 1; }
                        if bytes[i] == b')' { depth -= 1; }
                        if depth > 0 { i += 1; }
                    }
                    let expr_str = core::str::from_utf8(&bytes[start..i]).unwrap_or("");
                    if i < bytes.len() { i += 1; } // skip ')'
                    // Compile sub-expression
                    let mut sub_lexer = Lexer::new(expr_str);
                    // Swap lexer
                    let old_lexer = core::mem::replace(&mut self.lexer, sub_lexer);
                    self.parse_expr()?;
                    self.lexer = old_lexer;
                    self.emit(0, OP_TO_STRING)?;
                    if !first { self.emit(0, OP_STR_CONCAT)?; }
                    first = false;
                } else if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                    // $varname
                    let start = i;
                    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') { i += 1; }
                    let var_name = core::str::from_utf8(&bytes[start..i]).unwrap_or("");
                    let slot = self.vars.find_or_add(var_name).ok_or(CompileError::TooManyVariables)?;
                    self.emit(slot as u32, OP_LOAD)?;
                    self.emit(0, OP_TO_STRING)?;
                    if !first { self.emit(0, OP_STR_CONCAT)?; }
                    first = false;
                } else {
                    // Literal $
                    let sid = self.vm.strings.add_str("$");
                    self.emit(sid, OP_PUSH_STR)?;
                    if !first { self.emit(0, OP_STR_CONCAT)?; }
                    first = false;
                }
            } else {
                // Regular text segment
                let start = i;
                while i < bytes.len() && bytes[i] != b'$' { i += 1; }
                let segment = core::str::from_utf8(&bytes[start..i]).unwrap_or("");
                if !segment.is_empty() {
                    let sid = self.vm.strings.add_str(segment);
                    self.emit(sid, OP_PUSH_STR)?;
                    if !first { self.emit(0, OP_STR_CONCAT)?; }
                    first = false;
                }
            }
        }

        if first {
            // Chuỗi rỗng
            let sid = self.vm.strings.add_str("");
            self.emit(sid, OP_PUSH_STR)?;
        }
        Ok(())
    }

    // --- Parse expr / stmt forward declarations ---

    pub fn parse_expr(&mut self) -> Result<(), CompileError> {
        expr::parse_expr(self)
    }

    pub fn parse_block(&mut self) -> Result<(), CompileError> {
        stmt::parse_block(self)
    }

    fn parse_program(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_newlines();
            if self.lexer.kind() == TokenKind::Eof { break; }
            stmt::parse_stmt(self)?;
        }
        Ok(())
    }

    /// Biên dịch chương trình hoàn chỉnh
    pub fn compile(mut self) -> Result<(usize, VarTable, FuncTable), CompileError> {
        self.parse_program()?;
        self.emit(0, OP_HALT)?;
        let ptr = self.emit_ptr;
        Ok((ptr, self.vars, self.funcs))
    }
}

// === Public API ===

pub fn jl_compile(vm: &mut ForthVm, source: &str) -> Result<usize, CompileError> {
    vm.reset_code();
    let compiler = Compiler::new(vm, source);
    let (count, _, _) = compiler.compile()?;
    vm.pc = 0;
    Ok(count)
}

pub fn jl_run(vm: &mut ForthVm, source: &str) -> Result<crate::forthvm::vm::VmResult, CompileError> {
    jl_compile(vm, source)?;
    Ok(vm.run())
}
