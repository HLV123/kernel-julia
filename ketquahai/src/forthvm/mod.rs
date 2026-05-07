// ============================================================
// forthvm/mod.rs -- Module root (Stage 2)
// ============================================================

pub mod value;
pub mod opcode;
pub mod memory;
pub mod stack;
pub mod callstack;
pub mod registers;
pub mod frame_save;
pub mod syscall_bridge;
pub mod builtins;
pub mod handlers;
pub mod vm;
pub mod compiler;
pub mod lexer;
pub mod symbols;
pub mod disasm;
pub mod assembler;
pub mod state_persist;
pub mod repl;
pub mod demos;

pub use vm::{ForthVm, VmResult};
pub use compiler::jl_run;
pub use repl::{run_repl, run_file_oneshot};
