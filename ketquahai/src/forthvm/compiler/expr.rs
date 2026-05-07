// ============================================================
// compiler/expr.rs — Phân tích biểu thức (recursive descent)
// ============================================================

use alloc::string::String;
use alloc::format;
use crate::forthvm::opcode::*;
use crate::forthvm::lexer::TokenKind;
use crate::forthvm::builtins;
use super::{Compiler, CompileError};

/// Biểu thức chính (entry point)
pub fn parse_expr(c: &mut Compiler) -> Result<(), CompileError> {
    parse_ternary(c)
}

/// Ternary: expr ? expr : expr
fn parse_ternary(c: &mut Compiler) -> Result<(), CompileError> {
    parse_or(c)?;
    if c.lexer.kind() == TokenKind::Question {
        c.lexer.next(); // skip ?
        let jz = c.emit_jz_placeholder()?;
        parse_expr(c)?;
        if c.lexer.kind() != TokenKind::Colon {
            return Err(CompileError::UnexpectedToken(String::from("expected ':' in ternary")));
        }
        c.lexer.next(); // skip :
        let jmp = c.emit_jmp_placeholder()?;
        let else_addr = c.here();
        c.patch(jz, else_addr);
        parse_expr(c)?;
        let end_addr = c.here();
        c.patch(jmp, end_addr);
    }
    Ok(())
}

/// OR: a || b
fn parse_or(c: &mut Compiler) -> Result<(), CompileError> {
    parse_and(c)?;
    while c.lexer.kind() == TokenKind::Or {
        c.lexer.next();
        parse_and(c)?;
        c.emit(0, OP_OR)?;
    }
    Ok(())
}

/// AND: a && b
fn parse_and(c: &mut Compiler) -> Result<(), CompileError> {
    parse_not(c)?;
    while c.lexer.kind() == TokenKind::And {
        c.lexer.next();
        parse_not(c)?;
        c.emit(0, OP_AND)?;
    }
    Ok(())
}

/// NOT: !expr
fn parse_not(c: &mut Compiler) -> Result<(), CompileError> {
    if c.lexer.kind() == TokenKind::Not {
        c.lexer.next();
        parse_not(c)?;
        c.emit(0, OP_NOT)?;
        Ok(())
    } else {
        parse_comparison(c)
    }
}

/// Comparison: a == b, a != b, a < b, a > b, a <= b, a >= b
fn parse_comparison(c: &mut Compiler) -> Result<(), CompileError> {
    parse_bitwise_or(c)?;
    match c.lexer.kind() {
        TokenKind::Eq  => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_EQ)?; }
        TokenKind::Neq => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_NEQ)?; }
        TokenKind::Lt  => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_LT)?; }
        TokenKind::Gt  => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_GT)?; }
        TokenKind::Lte => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_LTE)?; }
        TokenKind::Gte => { c.lexer.next(); parse_bitwise_or(c)?; c.emit(0, OP_CMP_GTE)?; }
        _ => {}
    }
    Ok(())
}

/// Bitwise OR / XOR
fn parse_bitwise_or(c: &mut Compiler) -> Result<(), CompileError> {
    parse_bitwise_and(c)?;
    loop {
        match c.lexer.kind() {
            TokenKind::Pipe  => { c.lexer.next(); parse_bitwise_and(c)?; c.emit(0, OP_BOR)?; }
            TokenKind::Tilde => { c.lexer.next(); parse_bitwise_and(c)?; c.emit(0, OP_BXOR)?; }
            _ => break,
        }
    }
    Ok(())
}

/// Bitwise AND
fn parse_bitwise_and(c: &mut Compiler) -> Result<(), CompileError> {
    parse_shift(c)?;
    while c.lexer.kind() == TokenKind::Ampersand {
        c.lexer.next();
        parse_shift(c)?;
        c.emit(0, OP_BAND)?;
    }
    Ok(())
}

/// Shift: << >>
fn parse_shift(c: &mut Compiler) -> Result<(), CompileError> {
    parse_additive(c)?;
    loop {
        match c.lexer.kind() {
            TokenKind::Shl => { c.lexer.next(); parse_additive(c)?; c.emit(0, OP_SHL)?; }
            TokenKind::Shr => { c.lexer.next(); parse_additive(c)?; c.emit(0, OP_SHR)?; }
            _ => break,
        }
    }
    Ok(())
}

/// Additive: a + b, a - b
fn parse_additive(c: &mut Compiler) -> Result<(), CompileError> {
    parse_term(c)?;
    loop {
        match c.lexer.kind() {
            TokenKind::Plus  => { c.lexer.next(); parse_term(c)?; c.emit(0, OP_ADD)?; }
            TokenKind::Minus => { c.lexer.next(); parse_term(c)?; c.emit(0, OP_SUB)?; }
            _ => break,
        }
    }
    Ok(())
}

/// Term: a * b, a / b, a % b
fn parse_term(c: &mut Compiler) -> Result<(), CompileError> {
    parse_power(c)?;
    loop {
        match c.lexer.kind() {
            TokenKind::Star    => { c.lexer.next(); parse_power(c)?; c.emit(0, OP_MUL)?; }
            TokenKind::Slash   => { c.lexer.next(); parse_power(c)?; c.emit(0, OP_DIV)?; }
            TokenKind::Percent => { c.lexer.next(); parse_power(c)?; c.emit(0, OP_MOD)?; }
            _ => break,
        }
    }
    Ok(())
}

/// Power: a ^ b (right-associative)
fn parse_power(c: &mut Compiler) -> Result<(), CompileError> {
    parse_unary(c)?;
    if c.lexer.kind() == TokenKind::Caret {
        c.lexer.next();
        parse_power(c)?; // Right-associative
        c.emit(0, OP_POW)?;
    }
    Ok(())
}

/// Unary: -expr, +expr
fn parse_unary(c: &mut Compiler) -> Result<(), CompileError> {
    match c.lexer.kind() {
        TokenKind::Minus => {
            c.lexer.next();
            parse_postfix(c)?;
            c.emit(0, OP_NEG)?;
            Ok(())
        }
        TokenKind::Plus => {
            c.lexer.next();
            parse_postfix(c)
        }
        _ => parse_postfix(c),
    }
}

/// Postfix: expr[index], expr[index] = val (array access)
fn parse_postfix(c: &mut Compiler) -> Result<(), CompileError> {
    parse_factor(c)?;
    while c.lexer.kind() == TokenKind::LBracket {
        c.lexer.next(); // skip [
        parse_expr(c)?; // index
        if c.lexer.kind() != TokenKind::RBracket {
            return Err(CompileError::MissingBracket);
        }
        c.lexer.next(); // skip ]
        c.emit(0, OP_ARR_GET)?;
    }
    Ok(())
}

/// Factor: number, string, bool, nil, ident, call, (expr), [array]
fn parse_factor(c: &mut Compiler) -> Result<(), CompileError> {
    match c.lexer.kind() {
        TokenKind::Num | TokenKind::HexNum => {
            let val = c.lexer.current.num_val;
            c.emit(val as u32, OP_PUSH_INT)?;
            c.lexer.next();
            Ok(())
        }
        TokenKind::True => {
            c.emit(0, OP_PUSH_TRUE)?;
            c.lexer.next();
            Ok(())
        }
        TokenKind::False => {
            c.emit(0, OP_PUSH_FALSE)?;
            c.lexer.next();
            Ok(())
        }
        TokenKind::StringLit => {
            let raw = c.lexer.current.str_val.clone();
            c.lexer.next();
            if raw.contains('$') {
                c.compile_interpolated_string(&raw)?;
            } else {
                let sid = c.vm.strings.add_str(&raw);
                c.emit(sid, OP_PUSH_STR)?;
            }
            Ok(())
        }
        TokenKind::LParen => {
            c.lexer.next();
            parse_expr(c)?;
            if c.lexer.kind() != TokenKind::RParen {
                return Err(CompileError::MissingParen);
            }
            c.lexer.next();
            Ok(())
        }
        TokenKind::LBracket => {
            // Array literal: [1, 2, 3]
            c.lexer.next(); // skip [
            let mut count = 0u32;
            if c.lexer.kind() != TokenKind::RBracket {
                parse_expr(c)?;
                count += 1;
                while c.lexer.kind() == TokenKind::Comma {
                    c.lexer.next();
                    if c.lexer.kind() == TokenKind::RBracket { break; }
                    parse_expr(c)?;
                    count += 1;
                }
            }
            if c.lexer.kind() != TokenKind::RBracket {
                return Err(CompileError::MissingBracket);
            }
            c.lexer.next(); // skip ]
            c.emit(count, OP_ARR_LITERAL)?;
            Ok(())
        }
        TokenKind::Ident => {
            c.save_name();
            let name = String::from(c.saved_name());
            c.lexer.next();

            if c.lexer.kind() == TokenKind::LParen {
                // Function call
                parse_call(c, &name)?;
            } else {
                // Variable load
                let slot = c.vars.find_or_add(&name)
                    .ok_or(CompileError::TooManyVariables)?;
                c.emit(slot as u32, OP_LOAD)?;
            }
            Ok(())
        }
        _ => Err(CompileError::UnexpectedToken(
            format!("{:?} at line {} col {}", c.lexer.kind(), c.lexer.current.line, c.lexer.current.col)
        )),
    }
}

/// Parse function call: name(args...)
pub fn parse_call(c: &mut Compiler, name: &str) -> Result<(), CompileError> {
    c.lexer.next(); // skip (
    let mut arg_count = 0;
    while c.lexer.kind() != TokenKind::RParen && c.lexer.kind() != TokenKind::Eof {
        parse_expr(c)?;
        arg_count += 1;
        if c.lexer.kind() == TokenKind::Comma { c.lexer.next(); }
    }
    if c.lexer.kind() != TokenKind::RParen {
        return Err(CompileError::MissingParen);
    }
    c.lexer.next(); // skip )

    // Check built-in first
    if let Some(bi_id) = builtins::lookup_builtin(name) {
        c.emit(bi_id, OP_BUILTIN)?;
        return Ok(());
    }
    // Check sort!/reverse!/push!/pop!
    match name {
        "sort!" => { c.emit(builtins::BI_ARR_SORT, OP_BUILTIN)?; return Ok(()); }
        "reverse!" => { c.emit(builtins::BI_ARR_REVERSE, OP_BUILTIN)?; return Ok(()); }
        "push!" => { c.emit(builtins::BI_PUSH_BANG, OP_BUILTIN)?; return Ok(()); }
        "pop!" => { c.emit(builtins::BI_POP_BANG, OP_BUILTIN)?; return Ok(()); }
        _ => {}
    }

    // User-defined function
    if let Some((addr, _param_count)) = c.funcs.find(name) {
        c.emit(addr as u32, OP_CALL)?;
        Ok(())
    } else {
        Err(CompileError::FunctionNotFound(String::from(name)))
    }
}
