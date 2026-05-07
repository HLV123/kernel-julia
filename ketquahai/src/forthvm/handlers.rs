// ============================================================
// handlers.rs -- Bộ xử lý opcode (Stage 2: Value-based)
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::forthvm::opcode::*;
use crate::forthvm::value::{Value, StringPool, ArrayPool, format_value};
use crate::forthvm::stack::DataStack;
use crate::forthvm::callstack::CallStack;
use crate::forthvm::registers::Registers;
use crate::forthvm::frame_save::FrameStack;
use crate::forthvm::memory::VmMemory;
use crate::forthvm::syscall_bridge::SyscallBridge;
use crate::forthvm::builtins;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecError {
    StackUnderflow,
    StackOverflow,
    InvalidOpcode(u8),
    MemoryOutOfBounds,
    DivisionByZero,
    TypeError,
    CallStackOverflow,
    CallStackUnderflow,
    IndexOutOfBounds,
    BuiltinError,
    Halted,
}

/// Trạng thái VM — tham chiếu tất cả thành phần
pub struct VmState<'a> {
    pub memory:    &'a mut VmMemory,
    pub stack:     &'a mut DataStack,
    pub callstack: &'a mut CallStack,
    pub registers: &'a mut Registers,
    pub frames:    &'a mut FrameStack,
    pub bridge:    &'a mut SyscallBridge,
    pub strings:   &'a mut StringPool,
    pub arrays:    &'a mut ArrayPool,
    pub pc:        &'a mut usize,
    pub running:   &'a mut bool,
}

// --- Macro tiện lợi ---
macro_rules! pop {
    ($s:expr) => { $s.stack.pop().ok_or(ExecError::StackUnderflow)? };
}
macro_rules! push {
    ($s:expr, $v:expr) => {
        if !$s.stack.push($v) { return Err(ExecError::StackOverflow); }
    };
}
macro_rules! pop_int {
    ($s:expr) => {{
        let v = pop!($s);
        v.as_int().ok_or(ExecError::TypeError)?
    }};
}

// === Opcode Handlers ===

pub fn dispatch(state: &mut VmState, opcode: u8, arg: u32) -> Result<(), ExecError> {
    match opcode {
        // --- Stack & Constants ---
        OP_PUSH_INT   => { push!(state, Value::Int(arg as i32)); Ok(()) }
        OP_PUSH_TRUE  => { push!(state, Value::Bool(true)); Ok(()) }
        OP_PUSH_FALSE => { push!(state, Value::Bool(false)); Ok(()) }
        OP_PUSH_NIL   => { push!(state, Value::Nil); Ok(()) }
        OP_PUSH_STR   => { push!(state, Value::Str(arg)); Ok(()) }
        OP_DUP        => { if !state.stack.dup() { return Err(ExecError::StackUnderflow); } Ok(()) }
        OP_DROP       => { pop!(state); Ok(()) }
        OP_SWAP       => { if !state.stack.swap() { return Err(ExecError::StackUnderflow); } Ok(()) }

        // --- Arithmetic ---
        OP_ADD => {
            let b = pop!(state); let a = pop!(state);
            match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => push!(state, Value::Int(x.wrapping_add(*y))),
                _ => return Err(ExecError::TypeError),
            }
            Ok(())
        }
        OP_SUB => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a.wrapping_sub(b))); Ok(()) }
        OP_MUL => {
            let b = pop!(state); let a = pop!(state);
            match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => push!(state, Value::Int(x.wrapping_mul(*y))),
                // "abc" * 3 → repeat
                (Value::Str(sid), Value::Int(n)) => {
                    let s = state.strings.get(*sid).ok_or(ExecError::MemoryOutOfBounds)?;
                    let mut result = String::new();
                    for _ in 0..*n { result.push_str(s); }
                    let new_id = state.strings.add(result);
                    push!(state, Value::Str(new_id));
                }
                // "abc" * "def" → string concatenation
                (Value::Str(_), Value::Str(_)) => {
                    let sa = format_value(&a, state.strings, state.arrays);
                    let sb = format_value(&b, state.strings, state.arrays);
                    let mut result = sa;
                    result.push_str(&sb);
                    let sid = state.strings.add(result);
                    push!(state, Value::Str(sid));
                }
                _ => return Err(ExecError::TypeError),
            }
            Ok(())
        }
        OP_DIV => {
            let b = pop_int!(state); let a = pop_int!(state);
            if b == 0 { return Err(ExecError::DivisionByZero); }
            push!(state, Value::Int(a / b)); Ok(())
        }
        OP_MOD => {
            let b = pop_int!(state); let a = pop_int!(state);
            if b == 0 { return Err(ExecError::DivisionByZero); }
            push!(state, Value::Int(a % b)); Ok(())
        }
        OP_POW => {
            let b = pop_int!(state); let a = pop_int!(state);
            let mut result: i32 = 1;
            for _ in 0..b { result = result.wrapping_mul(a); }
            push!(state, Value::Int(result)); Ok(())
        }
        OP_NEG => {
            let a = pop_int!(state);
            push!(state, Value::Int(-a)); Ok(())
        }

        // --- Comparison ---
        OP_CMP_EQ => {
            let b = pop!(state); let a = pop!(state);
            let eq = match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => x == y,
                (Value::Bool(x), Value::Bool(y)) => x == y,
                (Value::Str(x), Value::Str(y)) => {
                    let sx = state.strings.get(*x).unwrap_or("");
                    let sy = state.strings.get(*y).unwrap_or("");
                    sx == sy
                }
                (Value::Nil, Value::Nil) => true,
                (Value::Int(n), Value::Bool(b)) | (Value::Bool(b), Value::Int(n)) => {
                    *n == (if *b { 1 } else { 0 })
                }
                _ => false,
            };
            push!(state, Value::Bool(eq)); Ok(())
        }
        OP_CMP_NEQ => {
            let b = pop!(state); let a = pop!(state);
            let eq = match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => x != y,
                (Value::Bool(x), Value::Bool(y)) => x != y,
                (Value::Str(x), Value::Str(y)) => {
                    state.strings.get(*x).unwrap_or("") != state.strings.get(*y).unwrap_or("")
                }
                _ => true,
            };
            push!(state, Value::Bool(eq)); Ok(())
        }
        OP_CMP_LT  => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Bool(a < b)); Ok(()) }
        OP_CMP_GT  => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Bool(a > b)); Ok(()) }
        OP_CMP_LTE => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Bool(a <= b)); Ok(()) }
        OP_CMP_GTE => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Bool(a >= b)); Ok(()) }

        // --- Logic ---
        OP_AND => { let b = pop!(state); let a = pop!(state); push!(state, Value::Bool(a.is_truthy() && b.is_truthy())); Ok(()) }
        OP_OR  => { let b = pop!(state); let a = pop!(state); push!(state, Value::Bool(a.is_truthy() || b.is_truthy())); Ok(()) }
        OP_NOT => { let a = pop!(state); push!(state, Value::Bool(!a.is_truthy())); Ok(()) }

        // --- Bitwise ---
        OP_BAND => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a & b)); Ok(()) }
        OP_BOR  => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a | b)); Ok(()) }
        OP_BXOR => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a ^ b)); Ok(()) }
        OP_SHL  => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a << (b & 31))); Ok(()) }
        OP_SHR  => { let b = pop_int!(state); let a = pop_int!(state); push!(state, Value::Int(a >> (b & 31))); Ok(()) }

        // --- Control flow ---
        OP_JMP => { *state.pc = arg as usize; Ok(()) }
        OP_JZ  => { let v = pop!(state); if !v.is_truthy() { *state.pc = arg as usize; } Ok(()) }
        OP_JNZ => { let v = pop!(state); if v.is_truthy() { *state.pc = arg as usize; } Ok(()) }
        OP_CALL => {
            if !state.callstack.push(*state.pc) { return Err(ExecError::CallStackOverflow); }
            *state.pc = arg as usize;
            Ok(())
        }
        OP_RET => {
            let addr = state.callstack.pop().ok_or(ExecError::CallStackUnderflow)?;
            *state.pc = addr;
            Ok(())
        }
        OP_HALT => { *state.running = false; Ok(()) }

        // --- Data ---
        OP_LOAD  => { let v = state.memory.data_load(arg as usize).unwrap_or(Value::Nil); push!(state, v); Ok(()) }
        OP_STORE => { let v = pop!(state); state.memory.data_store(arg as usize, v); Ok(()) }
        OP_PUSH_R => {
            let v = state.registers.get(arg as usize).cloned().unwrap_or(Value::Nil);
            push!(state, v); Ok(())
        }
        OP_POP_R => {
            let v = pop!(state);
            state.registers.set(arg as usize, v);
            Ok(())
        }

        // --- I/O ---
        OP_PRINT => {
            let v = pop!(state);
            state.bridge.print_value(&v, state.strings, state.arrays);
            Ok(())
        }
        OP_PRINT_NOLF => {
            let v = pop!(state);
            state.bridge.print_value_nolf(&v, state.strings, state.arrays);
            Ok(())
        }
        OP_READLINE => {
            // Đọc serial input (blocking spin)
            let line = read_serial_line();
            let sid = state.strings.add(line);
            push!(state, Value::Str(sid));
            Ok(())
        }

        // --- String ops ---
        OP_STR_CONCAT => {
            let b = pop!(state); let a = pop!(state);
            let sa = format_value(&a, state.strings, state.arrays);
            let sb = format_value(&b, state.strings, state.arrays);
            let mut result = sa;
            result.push_str(&sb);
            let sid = state.strings.add(result);
            push!(state, Value::Str(sid)); Ok(())
        }
        OP_STR_LEN => {
            if let Value::Str(id) = pop!(state) {
                let len = state.strings.get(id).map(|s| s.len()).unwrap_or(0);
                push!(state, Value::Int(len as i32));
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }
        OP_TO_STRING => {
            let v = pop!(state);
            let s = format_value(&v, state.strings, state.arrays);
            let sid = state.strings.add(s);
            push!(state, Value::Str(sid)); Ok(())
        }

        // --- Array ops ---
        OP_ARR_NEW => {
            let id = state.arrays.create();
            push!(state, Value::Array(id)); Ok(())
        }
        OP_ARR_LITERAL => {
            // arg = count, phần tử đã trên stack (đầu tiên ở sâu nhất)
            let count = arg as usize;
            let mut items = Vec::new();
            for _ in 0..count {
                items.push(pop!(state));
            }
            items.reverse();
            let id = state.arrays.create_with(items);
            push!(state, Value::Array(id)); Ok(())
        }
        OP_ARR_PUSH => {
            let val = pop!(state);
            let arr = pop!(state);
            if let Value::Array(id) = arr {
                state.arrays.push(id, val);
                push!(state, Value::Array(id));
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }
        OP_ARR_POP => {
            let arr = pop!(state);
            if let Value::Array(id) = arr {
                let v = state.arrays.pop(id).unwrap_or(Value::Nil);
                push!(state, v);
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }
        OP_ARR_GET => {
            let idx = pop_int!(state);
            let arr = pop!(state);
            if let Value::Array(id) = arr {
                let v = state.arrays.index_get(id, idx).ok_or(ExecError::IndexOutOfBounds)?;
                push!(state, v);
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }
        OP_ARR_SET => {
            let val = pop!(state);
            let idx = pop_int!(state);
            let arr = pop!(state);
            if let Value::Array(id) = arr {
                if !state.arrays.index_set(id, idx, val) { return Err(ExecError::IndexOutOfBounds); }
                push!(state, Value::Array(id));
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }
        OP_ARR_LEN => {
            let arr = pop!(state);
            if let Value::Array(id) = arr {
                let len = state.arrays.length(id).unwrap_or(0);
                push!(state, Value::Int(len as i32));
            } else { return Err(ExecError::TypeError); }
            Ok(())
        }

        // --- Built-in ---
        OP_BUILTIN => {
            let param_count = builtins::builtin_param_count(arg);
            let mut args_vec = Vec::new();
            for _ in 0..param_count {
                args_vec.push(pop!(state));
            }
            args_vec.reverse(); // Tham số đầu tiên ở vị trí 0
            match builtins::exec_builtin(arg, &args_vec, state.strings, state.arrays) {
                Ok(result) => { push!(state, result); Ok(()) }
                Err(_msg) => {
                    // Push error message as string
                    crate::println!("Error: {}", _msg);
                    push!(state, Value::Nil);
                    Ok(())
                }
            }
        }

        // --- State ---
        OP_SAVE    => { /* placeholder */ Ok(()) }
        OP_RESTORE => { /* placeholder */ Ok(()) }

        _ => Err(ExecError::InvalidOpcode(opcode)),
    }
}

/// Đọc 1 dòng từ serial (blocking)
fn read_serial_line() -> String {
    let mut buf = String::new();
    loop {
        let lsr: u8 = unsafe {
            let v: u8;
            core::arch::asm!("in al, dx", in("dx") 0x3FDu16, out("al") v, options(nomem, nostack));
            v
        };
        if lsr & 1 != 0 {
            let byte: u8 = unsafe {
                let v: u8;
                core::arch::asm!("in al, dx", in("dx") 0x3F8u16, out("al") v, options(nomem, nostack));
                v
            };
            match byte {
                b'\r' | b'\n' => return buf,
                b'\x08' | b'\x7f' => { buf.pop(); }
                b if b >= 0x20 => {
                    buf.push(b as char);
                    crate::print!("{}", b as char);
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}
