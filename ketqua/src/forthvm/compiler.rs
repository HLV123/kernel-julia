// ============================================================
// compiler.rs -- Trình biên dịch Julia (Phase 8)
// Phân tích cú pháp đệ quy → phát bytecode trực tiếp
// (chuyển từ 14-compiler.fs sang Rust)
// ============================================================

use alloc::string::String;
use crate::forthvm::opcode::*;
use crate::forthvm::lexer::{Lexer, TokenKind};
use crate::forthvm::symbols::{VarTable, FuncTable};
use crate::forthvm::vm::ForthVm;

/// Lỗi biên dịch
#[derive(Debug)]
pub enum CompileError {
    UnexpectedToken(String),
    MissingParen,
    MissingEnd,
    MissingFunctionName,
    MissingParamName,
    FunctionNotFound(String),
    TooManyVariables,
    TooManyFunctions,
    ProgramTooLarge,
}

/// Tối đa tham số cho 1 hàm
const MAX_PARAMS: usize = 8;

/// Trình biên dịch Julia → bytecode
pub struct Compiler<'a> {
    /// Lexer — nguồn cung cấp token
    lexer: Lexer,
    /// VM target — nơi phát bytecode
    vm: &'a mut ForthVm,
    /// Bảng biến
    vars: VarTable,
    /// Bảng hàm
    funcs: FuncTable,
    /// Con trỏ phát bytecode
    emit_ptr: usize,
    /// Bộ đệm lưu tên (bảo vệ khi gọi hàm lồng)
    saved_name: String,
    /// Bộ đệm tham số hàm
    param_slots: [usize; MAX_PARAMS],
    param_count: usize,
}

impl<'a> Compiler<'a> {
    /// Tạo compiler mới
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
        }
    }

    // --- Phát bytecode ---

    /// Vị trí hiện tại (tương đương jl-here)
    fn here(&self) -> usize {
        self.emit_ptr
    }

    /// Phát 1 lệnh đã đóng gói
    fn emit(&mut self, arg: u32, opcode: u8) -> Result<(), CompileError> {
        if self.emit_ptr >= PROG_SIZE {
            return Err(CompileError::ProgramTooLarge);
        }
        let _ = self.vm.memory.prog_write(self.emit_ptr, pack(arg, opcode));
        self.emit_ptr += 1;
        Ok(())
    }

    /// Phát JZ placeholder (sẽ backpatch sau)
    fn emit_jz_placeholder(&mut self) -> Result<usize, CompileError> {
        let patch = self.here();
        self.emit(0, OP_JZ)?;
        Ok(patch)
    }

    /// Phát JMP placeholder (sẽ backpatch sau)
    fn emit_jmp_placeholder(&mut self) -> Result<usize, CompileError> {
        let patch = self.here();
        self.emit(0, OP_JMP)?;
        Ok(patch)
    }

    /// Backpatch: ghi target address vào lệnh jump đã phát trước đó
    fn patch(&mut self, patch_addr: usize, target: usize) {
        if let Ok(cell) = self.vm.memory.prog_read(patch_addr) {
            let opcode = cell & 0xFF;
            let new_cell = ((target as u32) << 8) | opcode;
            let _ = self.vm.memory.prog_write(patch_addr, new_cell);
        }
    }

    // --- Bỏ qua dòng trống ---
    fn skip_newlines(&mut self) {
        while self.lexer.kind() == TokenKind::Newline {
            self.lexer.next();
        }
    }

    // --- Lưu tên token hiện tại ---
    fn save_name(&mut self) {
        self.saved_name = self.lexer.current.ident.clone();
    }

    // === Phân tích biểu thức (đệ quy giảm dần) ===

    /// Thừa số: số | biến | hàm(args) | (expr) | -expr
    fn parse_factor(&mut self) -> Result<(), CompileError> {
        match self.lexer.kind() {
            TokenKind::Num => {
                let val = self.lexer.num_val() as u32;
                self.emit(val, OP_PUSH)?;
                self.lexer.next();
                Ok(())
            }
            TokenKind::Ident => {
                self.save_name();
                self.lexer.next();
                if self.lexer.kind() == TokenKind::LParen {
                    // Gọi hàm
                    self.parse_call()?;
                } else {
                    // Đọc biến
                    let slot = self.vars.find_or_add(&self.saved_name.clone())
                        .ok_or(CompileError::TooManyVariables)?;
                    self.emit(slot as u32, OP_LOAD_DATA)?;
                }
                Ok(())
            }
            TokenKind::LParen => {
                self.lexer.next();
                self.parse_expr()?;
                if self.lexer.kind() != TokenKind::RParen {
                    return Err(CompileError::MissingParen);
                }
                self.lexer.next();
                Ok(())
            }
            TokenKind::Minus => {
                self.lexer.next();
                self.emit(0, OP_PUSH)?;
                self.parse_factor()?;
                self.emit(0, OP_SUB)?;
                Ok(())
            }
            _ => Err(CompileError::UnexpectedToken(
                alloc::format!("{:?}", self.lexer.kind())
            )),
        }
    }

    /// Tích: thừa_số { * thừa_số }
    fn parse_term(&mut self) -> Result<(), CompileError> {
        self.parse_factor()?;
        while self.lexer.kind() == TokenKind::Star {
            self.lexer.next();
            self.parse_factor()?;
            self.emit(0, OP_MUL)?;
        }
        Ok(())
    }

    /// Tổng: tích { (+|-) tích }
    fn parse_additive(&mut self) -> Result<(), CompileError> {
        self.parse_term()?;
        loop {
            match self.lexer.kind() {
                TokenKind::Plus => {
                    self.lexer.next();
                    self.parse_term()?;
                    self.emit(0, OP_ADD)?;
                }
                TokenKind::Minus => {
                    self.lexer.next();
                    self.parse_term()?;
                    self.emit(0, OP_SUB)?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// So sánh: tổng [ phép_so_sánh tổng ]
    fn parse_comparison(&mut self) -> Result<(), CompileError> {
        self.parse_additive()?;
        match self.lexer.kind() {
            TokenKind::Eq => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_CMP_EQ)?;
            }
            TokenKind::Neq => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_CMP_EQ)?;
                self.emit(0, OP_PUSH)?; // push 0
                self.emit(0, OP_CMP_EQ)?; // invert
            }
            TokenKind::Gt => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_CMP_GT)?;
            }
            TokenKind::Lt => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_SWAP)?;
                self.emit(0, OP_CMP_GT)?;
            }
            TokenKind::Gte => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_SWAP)?;
                self.emit(0, OP_CMP_GT)?;
                self.emit(0, OP_PUSH)?;
                self.emit(0, OP_CMP_EQ)?;
            }
            TokenKind::Lte => {
                self.lexer.next();
                self.parse_additive()?;
                self.emit(0, OP_CMP_GT)?;
                self.emit(0, OP_PUSH)?;
                self.emit(0, OP_CMP_EQ)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Biểu thức chính
    fn parse_expr(&mut self) -> Result<(), CompileError> {
        self.parse_comparison()
    }

    // === Phân tích câu lệnh ===

    /// Phân tích khối lệnh (dừng tại end / else / elseif / eof)
    fn parse_block(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_newlines();
            match self.lexer.kind() {
                TokenKind::End | TokenKind::Else | TokenKind::ElseIf | TokenKind::Eof => break,
                _ => self.parse_stmt()?,
            }
        }
        Ok(())
    }

    /// println(biểu_thức)
    fn parse_println(&mut self) -> Result<(), CompileError> {
        self.lexer.next(); // skip 'println'
        if self.lexer.kind() != TokenKind::LParen {
            return Err(CompileError::MissingParen);
        }
        self.lexer.next(); // skip '('
        self.parse_expr()?;
        if self.lexer.kind() != TokenKind::RParen {
            return Err(CompileError::MissingParen);
        }
        self.lexer.next(); // skip ')'
        self.emit(0, OP_PRINT)?;
        Ok(())
    }

    /// if điều_kiện ... else ... end
    fn parse_if(&mut self) -> Result<(), CompileError> {
        self.lexer.next(); // skip 'if'
        self.parse_expr()?;
        let jz_patch = self.emit_jz_placeholder()?;
        self.skip_newlines();
        self.parse_block()?;

        if self.lexer.kind() == TokenKind::Else {
            self.lexer.next(); // skip 'else'
            let jmp_patch = self.emit_jmp_placeholder()?;
            let else_target = self.here();
            self.patch(jz_patch, else_target);
            self.skip_newlines();
            self.parse_block()?;
            let end_target = self.here();
            self.patch(jmp_patch, end_target);
        } else {
            let end_target = self.here();
            self.patch(jz_patch, end_target);
        }

        if self.lexer.kind() != TokenKind::End {
            return Err(CompileError::MissingEnd);
        }
        self.lexer.next(); // skip 'end'
        Ok(())
    }

    /// while điều_kiện ... end
    fn parse_while(&mut self) -> Result<(), CompileError> {
        self.lexer.next(); // skip 'while'
        let loop_start = self.here();
        self.parse_expr()?;
        let jz_patch = self.emit_jz_placeholder()?;
        self.skip_newlines();
        self.parse_block()?;
        self.emit(loop_start as u32, OP_JMP)?;
        let end_target = self.here();
        self.patch(jz_patch, end_target);

        if self.lexer.kind() != TokenKind::End {
            return Err(CompileError::MissingEnd);
        }
        self.lexer.next(); // skip 'end'
        Ok(())
    }

    /// return biểu_thức
    fn parse_return(&mut self) -> Result<(), CompileError> {
        self.lexer.next(); // skip 'return'
        self.parse_expr()?;
        self.emit(0, OP_RET)?;
        Ok(())
    }

    /// Gọi hàm: tra tên → đẩy tham số → CALL
    fn parse_call(&mut self) -> Result<(), CompileError> {
        let func_name = self.saved_name.clone();
        self.lexer.next(); // skip '('

        // Parse arguments
        while self.lexer.kind() != TokenKind::RParen {
            self.parse_expr()?;
            if self.lexer.kind() == TokenKind::Comma {
                self.lexer.next(); // skip ','
            }
        }
        self.lexer.next(); // skip ')'

        // Tìm hàm
        let (code_addr, _param_count) = self.funcs.find(&func_name)
            .ok_or_else(|| CompileError::FunctionNotFound(func_name))?;
        self.emit(code_addr as u32, OP_CALL)?;
        Ok(())
    }

    /// Định nghĩa hàm
    fn parse_function(&mut self) -> Result<(), CompileError> {
        self.lexer.next(); // skip 'function'

        // Tên hàm
        if self.lexer.kind() != TokenKind::Ident {
            return Err(CompileError::MissingFunctionName);
        }
        let func_name = self.lexer.current.ident.clone();
        self.lexer.next();

        // Jump qua thân hàm
        let jmp_patch = self.emit_jmp_placeholder()?;
        let func_entry = self.here();

        // Parse tham số
        self.param_count = 0;
        if self.lexer.kind() != TokenKind::LParen {
            return Err(CompileError::MissingParen);
        }
        self.lexer.next(); // skip '('
        while self.lexer.kind() != TokenKind::RParen {
            if self.lexer.kind() != TokenKind::Ident {
                return Err(CompileError::MissingParamName);
            }
            let param_name = self.lexer.current.ident.clone();
            let slot = self.vars.find_or_add(&param_name)
                .ok_or(CompileError::TooManyVariables)?;
            if self.param_count < MAX_PARAMS {
                self.param_slots[self.param_count] = slot;
                self.param_count += 1;
            }
            self.lexer.next();
            if self.lexer.kind() == TokenKind::Comma {
                self.lexer.next(); // skip ','
            }
        }
        self.lexer.next(); // skip ')'

        // Emit prologue: pop tham số từ stack vào Data slots
        // Đảo thứ tự: tham số cuối cùng ở đỉnh stack
        for i in (0..self.param_count).rev() {
            self.emit(self.param_slots[i] as u32, OP_STORE_DATA)?;
        }

        // Đăng ký hàm
        if !self.funcs.add(&func_name, func_entry, self.param_count) {
            return Err(CompileError::TooManyFunctions);
        }

        // Parse thân hàm
        self.skip_newlines();
        self.parse_block()?;

        // Default return 0
        self.emit(0, OP_PUSH)?;
        self.emit(0, OP_RET)?;

        // Backpatch jump
        let after_func = self.here();
        self.patch(jmp_patch, after_func);

        if self.lexer.kind() != TokenKind::End {
            return Err(CompileError::MissingEnd);
        }
        self.lexer.next(); // skip 'end'
        Ok(())
    }

    /// Câu lệnh
    fn parse_stmt(&mut self) -> Result<(), CompileError> {
        match self.lexer.kind() {
            TokenKind::Newline => {
                self.lexer.next();
                Ok(())
            }
            TokenKind::Eof => Ok(()),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Function => self.parse_function(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Println => self.parse_println(),
            TokenKind::Ident => {
                self.save_name();
                self.lexer.next();
                match self.lexer.kind() {
                    TokenKind::Assign => {
                        // Gán biến
                        let var_name = self.saved_name.clone();
                        let slot = self.vars.find_or_add(&var_name)
                            .ok_or(CompileError::TooManyVariables)?;
                        self.lexer.next(); // skip '='
                        self.parse_expr()?;
                        self.emit(slot as u32, OP_STORE_DATA)?;
                        Ok(())
                    }
                    TokenKind::LParen => {
                        // Gọi hàm (không dùng return value)
                        self.parse_call()?;
                        self.emit(0, OP_DROP)?;
                        Ok(())
                    }
                    _ => Err(CompileError::UnexpectedToken(
                        alloc::format!("after ident '{}'", self.saved_name)
                    )),
                }
            }
            _ => Err(CompileError::UnexpectedToken(
                alloc::format!("{:?}", self.lexer.kind())
            )),
        }
    }

    /// Phân tích toàn bộ chương trình
    fn parse_program(&mut self) -> Result<(), CompileError> {
        loop {
            self.skip_newlines();
            if self.lexer.kind() == TokenKind::Eof {
                break;
            }
            self.parse_stmt()?;
        }
        Ok(())
    }

    /// Biên dịch chương trình Julia
    /// Trả về số lệnh bytecode đã phát
    pub fn compile(mut self) -> Result<usize, CompileError> {
        self.parse_program()?;
        self.emit(0, OP_HALT)?;
        Ok(self.emit_ptr)
    }
}

// === API tiện lợi ===

/// Biên dịch mã Julia vào VM
pub fn jl_compile(vm: &mut ForthVm, source: &str) -> Result<usize, CompileError> {
    vm.reset();
    let compiler = Compiler::new(vm, source);
    let count = compiler.compile()?;
    vm.pc = 0;
    Ok(count)
}

/// Biên dịch và chạy mã Julia
pub fn jl_run(vm: &mut ForthVm, source: &str) -> Result<crate::forthvm::vm::VmResult, CompileError> {
    jl_compile(vm, source)?;
    Ok(vm.run())
}

/// Biên dịch và hiển thị bytecode (disassemble)
pub fn jl_disasm(vm: &mut ForthVm, source: &str) -> Result<(), CompileError> {
    let count = jl_compile(vm, source)?;
    vm.find_prog_end();
    let text = crate::forthvm::disasm::disasm(&vm.memory, vm.prog_end);
    crate::print!("{}", text);
    let _ = count;
    Ok(())
}
