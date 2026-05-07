// ============================================================
// compiler/stmt.rs — Phân tích câu lệnh
// if/elseif/else, while, for, function, break, continue,
// println, print, assignment, compound assign, include
// ============================================================

use alloc::string::String;
use alloc::format;
use crate::forthvm::opcode::*;
use crate::forthvm::lexer::TokenKind;
use crate::forthvm::symbols::MAX_PARAMS;
use super::{Compiler, CompileError};
use super::expr;

/// Phân tích khối lệnh (dừng tại end/else/elseif/eof)
pub fn parse_block(c: &mut Compiler) -> Result<(), CompileError> {
    loop {
        c.skip_newlines();
        match c.lexer.kind() {
            TokenKind::End | TokenKind::Else | TokenKind::ElseIf | TokenKind::Eof => break,
            _ => parse_stmt(c)?,
        }
    }
    Ok(())
}

/// Câu lệnh đơn
pub fn parse_stmt(c: &mut Compiler) -> Result<(), CompileError> {
    match c.lexer.kind() {
        TokenKind::Newline => { c.lexer.next(); Ok(()) }
        TokenKind::Eof => Ok(()),
        TokenKind::If => parse_if(c),
        TokenKind::While => parse_while(c),
        TokenKind::For => parse_for(c),
        TokenKind::Function => parse_function(c),
        TokenKind::Return => parse_return(c),
        TokenKind::Println => parse_println(c),
        TokenKind::Print => parse_print(c),
        TokenKind::Break => parse_break(c),
        TokenKind::Continue => parse_continue(c),
        TokenKind::Local => parse_local(c),
        TokenKind::Include => parse_include(c),
        TokenKind::Ident => parse_ident_stmt(c),
        _ => {
            // Expression statement (auto-print in REPL)
            c.is_expr_stmt = true;
            c.parse_expr()?;
            Ok(())
        }
    }
}

fn parse_if(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next(); // skip 'if'
    c.parse_expr()?;
    let mut current_jz = c.emit_jz_placeholder()?;
    c.skip_newlines();
    parse_block(c)?;

    let mut end_patches: alloc::vec::Vec<usize> = alloc::vec::Vec::new();

    // Handle elseif chain
    while c.lexer.kind() == TokenKind::ElseIf {
        c.lexer.next(); // skip 'elseif'
        let jmp = c.emit_jmp_placeholder()?;
        end_patches.push(jmp);
        let else_target = c.here();
        c.patch(current_jz, else_target);
        c.parse_expr()?;
        current_jz = c.emit_jz_placeholder()?;
        c.skip_newlines();
        parse_block(c)?;
    }

    if c.lexer.kind() == TokenKind::Else {
        c.lexer.next();
        let jmp = c.emit_jmp_placeholder()?;
        end_patches.push(jmp);
        let else_target = c.here();
        c.patch(current_jz, else_target);
        c.skip_newlines();
        parse_block(c)?;
    } else {
        let end_target = c.here();
        c.patch(current_jz, end_target);
    }

    let end_addr = c.here();
    for p in &end_patches {
        c.patch(*p, end_addr);
    }

    if c.lexer.kind() != TokenKind::End {
        return Err(CompileError::MissingEnd);
    }
    c.lexer.next();
    Ok(())
}

/// while condition ... end
fn parse_while(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next(); // skip 'while'
    let loop_start = c.here();

    // Push loop context
    c.loop_stack.push(super::LoopCtx {
        break_patches: alloc::vec::Vec::new(),
        continue_patches: alloc::vec::Vec::new(),
        continue_target: Some(loop_start),
    });

    c.parse_expr()?;
    let jz_patch = c.emit_jz_placeholder()?;
    c.skip_newlines();
    parse_block(c)?;
    c.emit(loop_start as u32, OP_JMP)?;
    let end_target = c.here();
    c.patch(jz_patch, end_target);

    // Patch all break jumps
    if let Some(ctx) = c.loop_stack.pop() {
        for p in &ctx.break_patches {
            c.patch(*p, end_target);
        }
    }

    if c.lexer.kind() != TokenKind::End {
        return Err(CompileError::MissingEnd);
    }
    c.lexer.next();
    Ok(())
}

/// for i = start:end ... end
/// for i = start:step:end ... end
fn parse_for(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next(); // skip 'for'

    if c.lexer.kind() != TokenKind::Ident {
        return Err(CompileError::UnexpectedToken(String::from("expected variable in for")));
    }
    let var_name = c.lexer.current.str_val.clone();
    c.lexer.next();

    if c.lexer.kind() != TokenKind::Assign {
        return Err(CompileError::UnexpectedToken(String::from("expected '=' in for")));
    }
    c.lexer.next(); // skip '='

    // Parse start value
    c.parse_expr()?;
    let var_slot = c.vars.find_or_add(&var_name).ok_or(CompileError::TooManyVariables)?;
    c.emit(var_slot as u32, OP_STORE)?;

    // Expect ':'
    if c.lexer.kind() != TokenKind::Colon {
        return Err(CompileError::UnexpectedToken(String::from("expected ':' in range")));
    }
    c.lexer.next();

    // Parse second value
    c.parse_expr()?;

    // Check for step: start:step:end
    let has_step = c.lexer.kind() == TokenKind::Colon;
    let step_slot;
    let end_slot;

    if has_step {
        // start:step:end — giá trị vừa parse là step
        step_slot = c.vars.find_or_add("__for_step").ok_or(CompileError::TooManyVariables)?;
        c.emit(step_slot as u32, OP_STORE)?;
        c.lexer.next(); // skip ':'
        c.parse_expr()?;
        end_slot = c.vars.find_or_add("__for_end").ok_or(CompileError::TooManyVariables)?;
        c.emit(end_slot as u32, OP_STORE)?;
    } else {
        // start:end — step = 1
        end_slot = c.vars.find_or_add("__for_end").ok_or(CompileError::TooManyVariables)?;
        c.emit(end_slot as u32, OP_STORE)?;
        step_slot = c.vars.find_or_add("__for_step").ok_or(CompileError::TooManyVariables)?;
        c.emit(1u32, OP_PUSH_INT)?;
        c.emit(step_slot as u32, OP_STORE)?;
    }

    let loop_start = c.here();
    c.loop_stack.push(super::LoopCtx {
        break_patches: alloc::vec::Vec::new(),
        continue_patches: alloc::vec::Vec::new(),
        continue_target: None, // Will be resolved to the increment block
    });

    // Condition: if step > 0 then i <= end, else i >= end
    // Simplified: always check i <= end for positive step, i >= end for negative
    c.emit(step_slot as u32, OP_LOAD)?;
    c.emit(0u32, OP_PUSH_INT)?;
    c.emit(0, OP_CMP_GT)?;
    let jz_dir = c.emit_jz_placeholder()?;

    // Positive step: i <= end
    c.emit(var_slot as u32, OP_LOAD)?;
    c.emit(end_slot as u32, OP_LOAD)?;
    c.emit(0, OP_CMP_LTE)?;
    let jmp_merge = c.emit_jmp_placeholder()?;

    // Negative step: i >= end
    let neg_addr = c.here();
    c.patch(jz_dir, neg_addr);
    c.emit(var_slot as u32, OP_LOAD)?;
    c.emit(end_slot as u32, OP_LOAD)?;
    c.emit(0, OP_CMP_GTE)?;

    let merge_addr = c.here();
    c.patch(jmp_merge, merge_addr);

    let jz_exit = c.emit_jz_placeholder()?;

    c.skip_newlines();
    parse_block(c)?;

    // Increment block
    let increment_start = c.here();

    // Increment: i += step
    c.emit(var_slot as u32, OP_LOAD)?;
    c.emit(step_slot as u32, OP_LOAD)?;
    c.emit(0, OP_ADD)?;
    c.emit(var_slot as u32, OP_STORE)?;

    c.emit(loop_start as u32, OP_JMP)?;
    let end_target = c.here();
    c.patch(jz_exit, end_target);

    if let Some(ctx) = c.loop_stack.pop() {
        for p in &ctx.break_patches {
            c.patch(*p, end_target);
        }
        for p in &ctx.continue_patches {
            c.patch(*p, increment_start);
        }
    }

    if c.lexer.kind() != TokenKind::End {
        return Err(CompileError::MissingEnd);
    }
    c.lexer.next();
    Ok(())
}

/// function name(params...) ... end
fn parse_function(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next(); // skip 'function'
    if c.lexer.kind() != TokenKind::Ident {
        return Err(CompileError::MissingFunctionName);
    }
    let func_name = c.lexer.current.str_val.clone();
    c.lexer.next();

    let jmp_patch = c.emit_jmp_placeholder()?;
    let func_entry = c.here();

    // Parse params
    c.param_count = 0;
    if c.lexer.kind() != TokenKind::LParen {
        return Err(CompileError::MissingParen);
    }
    c.lexer.next();
    while c.lexer.kind() != TokenKind::RParen && c.lexer.kind() != TokenKind::Eof {
        if c.lexer.kind() != TokenKind::Ident {
            return Err(CompileError::MissingParamName);
        }
        let pname = c.lexer.current.str_val.clone();
        let slot = c.vars.find_or_add(&pname).ok_or(CompileError::TooManyVariables)?;
        if c.param_count < MAX_PARAMS {
            c.param_slots[c.param_count] = slot;
            c.param_count += 1;
        }
        c.lexer.next();
        if c.lexer.kind() == TokenKind::Comma { c.lexer.next(); }
    }
    if c.lexer.kind() != TokenKind::RParen { return Err(CompileError::MissingParen); }
    c.lexer.next();

    // Emit prologue: pop params vào slots (ngược)
    for i in (0..c.param_count).rev() {
        c.emit(c.param_slots[i] as u32, OP_STORE)?;
    }

    let mut ps = [0usize; MAX_PARAMS];
    ps[..c.param_count].copy_from_slice(&c.param_slots[..c.param_count]);
    c.funcs.add(&func_name, func_entry, c.param_count, ps);

    c.skip_newlines();
    parse_block(c)?;

    // Default return nil
    c.emit(0, OP_PUSH_NIL)?;
    c.emit(0, OP_RET)?;

    let after = c.here();
    c.patch(jmp_patch, after);

    if c.lexer.kind() != TokenKind::End { return Err(CompileError::MissingEnd); }
    c.lexer.next();
    Ok(())
}

/// return expr
fn parse_return(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    if c.lexer.kind() != TokenKind::Newline && c.lexer.kind() != TokenKind::Eof
        && c.lexer.kind() != TokenKind::End {
        c.parse_expr()?;
    } else {
        c.emit(0, OP_PUSH_NIL)?;
    }
    c.emit(0, OP_RET)?;
    Ok(())
}

/// println(exprs...)
fn parse_println(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    if c.lexer.kind() != TokenKind::LParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    if c.lexer.kind() == TokenKind::RParen {
        // println() — in dòng trống
        let sid = c.vm.strings.add_str("");
        c.emit(sid, OP_PUSH_STR)?;
        c.emit(0, OP_PRINT)?;
    } else {
        c.parse_expr()?;
        // Nhiều tham số: println("x = ", x, " y = ", y)
        while c.lexer.kind() == TokenKind::Comma {
            c.lexer.next();
            c.emit(0, OP_PRINT_NOLF)?;
            c.parse_expr()?;
        }
        c.emit(0, OP_PRINT)?;
    }
    if c.lexer.kind() != TokenKind::RParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    Ok(())
}

/// print(exprs...)
fn parse_print(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    if c.lexer.kind() != TokenKind::LParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    c.parse_expr()?;
    while c.lexer.kind() == TokenKind::Comma {
        c.lexer.next();
        c.emit(0, OP_PRINT_NOLF)?;
        c.parse_expr()?;
    }
    c.emit(0, OP_PRINT_NOLF)?;
    if c.lexer.kind() != TokenKind::RParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    Ok(())
}

/// break
fn parse_break(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    let p = c.emit_jmp_placeholder()?;
    if let Some(ctx) = c.loop_stack.last_mut() {
        ctx.break_patches.push(p);
        Ok(())
    } else {
        Err(CompileError::UnexpectedToken(String::from("break outside loop")))
    }
}

/// continue
fn parse_continue(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    if c.loop_stack.is_empty() {
        return Err(CompileError::UnexpectedToken(String::from("continue outside loop")));
    }
    let continue_target = c.loop_stack.last().and_then(|ctx| ctx.continue_target);
    if let Some(target) = continue_target {
        c.emit(target as u32, OP_JMP)?;
    } else {
        let p = c.emit_jmp_placeholder()?;
        c.loop_stack.last_mut().unwrap().continue_patches.push(p);
    }
    Ok(())
}

/// local var = expr
fn parse_local(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next(); // skip 'local'
    if c.lexer.kind() != TokenKind::Ident {
        return Err(CompileError::UnexpectedToken(String::from("expected variable after local")));
    }
    let name = c.lexer.current.str_val.clone();
    c.lexer.next();
    let slot = c.vars.find_or_add(&name).ok_or(CompileError::TooManyVariables)?;
    if c.lexer.kind() == TokenKind::Assign {
        c.lexer.next();
        c.parse_expr()?;
    } else {
        c.emit(0, OP_PUSH_NIL)?;
    }
    c.emit(slot as u32, OP_STORE)?;
    Ok(())
}

/// include("path")
fn parse_include(c: &mut Compiler) -> Result<(), CompileError> {
    c.lexer.next();
    if c.lexer.kind() != TokenKind::LParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    if c.lexer.kind() != TokenKind::StringLit {
        return Err(CompileError::InvalidString);
    }
    let _path = c.lexer.current.str_val.clone();
    c.lexer.next();
    if c.lexer.kind() != TokenKind::RParen { return Err(CompileError::MissingParen); }
    c.lexer.next();
    // Include is handled at REPL level, not here
    // We just emit a no-op
    Ok(())
}

/// Identifier statement: assignment, compound assign, function call, or array set
fn parse_ident_stmt(c: &mut Compiler) -> Result<(), CompileError> {
    c.save_name();
    let name = String::from(c.saved_name());
    c.lexer.next();

    match c.lexer.kind() {
        TokenKind::Assign => {
            let slot = c.vars.find_or_add(&name).ok_or(CompileError::TooManyVariables)?;
            c.lexer.next();
            c.parse_expr()?;
            c.emit(slot as u32, OP_STORE)?;
            Ok(())
        }
        // Compound assignment: +=, -=, *=, /=, %=
        TokenKind::PlusEq | TokenKind::MinusEq | TokenKind::StarEq |
        TokenKind::SlashEq | TokenKind::PercentEq => {
            let op = c.lexer.kind();
            let slot = c.vars.find_or_add(&name).ok_or(CompileError::TooManyVariables)?;
            c.lexer.next();
            c.emit(slot as u32, OP_LOAD)?;
            c.parse_expr()?;
            match op {
                TokenKind::PlusEq    => c.emit(0, OP_ADD)?,
                TokenKind::MinusEq   => c.emit(0, OP_SUB)?,
                TokenKind::StarEq    => c.emit(0, OP_MUL)?,
                TokenKind::SlashEq   => c.emit(0, OP_DIV)?,
                TokenKind::PercentEq => c.emit(0, OP_MOD)?,
                _ => {}
            }
            c.emit(slot as u32, OP_STORE)?;
            Ok(())
        }
        TokenKind::LParen => {
            // Function call statement (drop return value)
            expr::parse_call(c, &name)?;
            c.emit(0, OP_DROP)?;
            Ok(())
        }
        TokenKind::LBracket => {
            // Array set: name[index] = val
            let slot = c.vars.find_or_add(&name).ok_or(CompileError::TooManyVariables)?;
            c.emit(slot as u32, OP_LOAD)?;
            c.lexer.next(); // skip [
            c.parse_expr()?; // index
            if c.lexer.kind() != TokenKind::RBracket {
                return Err(CompileError::MissingBracket);
            }
            c.lexer.next(); // skip ]
            if c.lexer.kind() != TokenKind::Assign {
                return Err(CompileError::UnexpectedToken(String::from("expected '=' in array set")));
            }
            c.lexer.next(); // skip =
            c.parse_expr()?; // value
            c.emit(0, OP_ARR_SET)?;
            c.emit(0, OP_DROP)?; // drop the returned array ref
            Ok(())
        }
        _ => {
            Err(CompileError::UnexpectedToken(
                format!("after '{}': {:?}", name, c.lexer.kind())
            ))
        }
    }
}
