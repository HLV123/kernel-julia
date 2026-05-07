// ============================================================
// forthvm/mod.rs -- Module root
// Julia-Forth VM -- Phase 25: Userland Runtime
//
// Chuyển đổi toàn bộ ForthVM (8 phases, 18 file Forth)
// sang Rust no_std, chạy như kernel module trong MyKernel.
//
// Cấu trúc module:
//   opcode        — Hằng số, mã lệnh (00-constants.fs)
//   memory        — Bố trí bộ nhớ (01-memory.fs)
//   stack         — Ngăn xếp dữ liệu (02-stack.fs)
//   callstack     — Ngăn xếp gọi hàm (05-callstack.fs)
//   registers     — Thanh ghi R0-R7 (03-registers.fs)
//   frame_save    — Frame cho đệ quy đôi (Phase 9 mới)
//   syscall_bridge— Cầu nối VM → kernel
//   handlers      — Bộ xử lý lệnh (07-handlers.fs)
//   vm            — Lõi máy ảo (04-program.fs + 08-dispatch.fs)
//   disasm        — Dịch ngược (09-disasm.fs)
//   assembler     — Hợp dịch văn bản (11-assembler.fs)
//   lexer         — Phân tích từ tố Julia (12-lexer.fs)
//   symbols       — Bảng ký hiệu (13-symbols.fs)
//   compiler      — Biên dịch Julia (14-compiler.fs)
//   state_persist — Lưu/khôi phục trạng thái (Phase 7 cải tiến)
//   demos         — Chương trình mẫu (15-demos.fs)
// ============================================================

pub mod opcode;
pub mod memory;
pub mod stack;
pub mod callstack;
pub mod registers;
pub mod frame_save;
pub mod syscall_bridge;
pub mod handlers;
pub mod vm;
pub mod disasm;
pub mod assembler;
pub mod lexer;
pub mod symbols;
pub mod compiler;
pub mod state_persist;
pub mod demos;

// Re-export các types hay dùng
pub use vm::{ForthVm, VmResult};
pub use compiler::{jl_compile, jl_run};
pub use assembler::assemble_into;
pub use syscall_bridge::SyscallBridge;
