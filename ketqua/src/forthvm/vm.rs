// ============================================================
// vm.rs -- Lõi máy ảo (VM Core)
// Quản lý trạng thái, fetch lệnh, dispatch loop
// (chuyển từ 04-program.fs + 08-dispatch.fs sang Rust)
// ============================================================

use crate::forthvm::opcode::*;
use crate::forthvm::memory::VmMemory;
use crate::forthvm::stack::DataStack;
use crate::forthvm::callstack::CallStack;
use crate::forthvm::registers::Registers;
use crate::forthvm::frame_save::FrameStack;
use crate::forthvm::syscall_bridge::SyscallBridge;
use crate::forthvm::handlers::{self, ExecError, VmState};

/// Kết quả sau khi VM chạy xong
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmResult {
    /// VM dừng bình thường (OP_HALT)
    Halted,
    /// VM gặp lỗi
    Error(ExecError),
    /// VM chưa dừng — đã yield sau N lệnh (async mode)
    Yielded,
}

/// Máy ảo Julia-Forth — chứa toàn bộ trạng thái
pub struct ForthVm {
    /// Bộ nhớ chương trình (bytecode + data + heap)
    pub memory: VmMemory,
    /// Ngăn xếp dữ liệu
    pub stack: DataStack,
    /// Ngăn xếp lời gọi hàm
    pub callstack: CallStack,
    /// 8 thanh ghi đa dụng R0–R7
    pub registers: Registers,
    /// Frame stack cho đệ quy đôi (Phase 9)
    pub frames: FrameStack,
    /// Lớp trung gian VM → Kernel
    pub bridge: SyscallBridge,

    /// Con trỏ lệnh (Program Counter)
    pub pc: usize,
    /// Cờ chạy: true = đang chạy, false = đã dừng
    pub running: bool,
    /// Vị trí kết thúc chương trình (cho debugger)
    pub prog_end: usize,
}

impl ForthVm {
    /// Tạo VM mới với bridge mặc định (kernel mode)
    pub fn new() -> Self {
        ForthVm {
            memory: VmMemory::new(),
            stack: DataStack::new(),
            callstack: CallStack::new(),
            registers: Registers::new(),
            frames: FrameStack::new(),
            bridge: SyscallBridge::new(),
            pc: 0,
            running: false,
            prog_end: 0,
        }
    }

    /// Tạo VM mới với bridge tuỳ chỉnh
    pub fn with_bridge(bridge: SyscallBridge) -> Self {
        ForthVm {
            memory: VmMemory::new(),
            stack: DataStack::new(),
            callstack: CallStack::new(),
            registers: Registers::new(),
            frames: FrameStack::new(),
            bridge,
            pc: 0,
            running: false,
            prog_end: 0,
        }
    }

    /// Khởi tạo lại toàn bộ VM (giống vm-init trong Forth)
    pub fn reset(&mut self) {
        self.pc = 0;
        self.stack.reset();
        self.callstack.reset();
        self.registers.reset();
        self.frames.reset();
        self.memory.reset();
        self.running = false;
        self.prog_end = 0;
    }

    /// Nạp lệnh tiếp theo tại pc, tách mã lệnh và tham số
    /// Mỗi cell đóng gói: (arg << 8) | opcode
    ///   - 8 bit thấp → mã lệnh (opcode)
    ///   - Các bit cao → tham số (arg)
    pub fn fetch(&mut self) -> Result<(u8, u32), ExecError> {
        let cell = self.memory.prog_read(self.pc)
            .map_err(|_| ExecError::MemoryOutOfBounds)?;
        self.pc += 1;
        let opcode = unpack_opcode(cell);
        let arg = unpack_arg(cell);
        Ok((opcode, arg))
    }

    /// Phân phối lệnh — nhận mã lệnh, gọi handler đúng
    /// (chuyển từ dispatch trong 08-dispatch.fs)
    pub fn dispatch(&mut self, opcode: u8, arg: u32) -> Result<(), ExecError> {
        // Tạo VmState tham chiếu tới tất cả các thành phần
        let mut state = VmState {
            memory:    &mut self.memory,
            stack:     &mut self.stack,
            callstack: &mut self.callstack,
            registers: &mut self.registers,
            frames:    &mut self.frames,
            bridge:    &mut self.bridge,
            pc:        &mut self.pc,
            running:   &mut self.running,
        };

        match opcode {
            OP_PUSH       => handlers::op_push(&mut state, arg),
            OP_ADD        => handlers::op_add(&mut state),
            OP_SUB        => handlers::op_sub(&mut state),
            OP_MUL        => handlers::op_mul(&mut state),
            OP_PUSH_R     => handlers::op_push_r(&mut state, arg),
            OP_POP_R      => handlers::op_pop_r(&mut state, arg),
            OP_PRINT      => handlers::op_print(&mut state),
            OP_JMP        => handlers::op_jmp(&mut state, arg),
            OP_JZ         => handlers::op_jz(&mut state, arg),
            OP_JGT        => handlers::op_jgt(&mut state, arg),
            OP_CALL       => handlers::op_call(&mut state, arg),
            OP_RET        => handlers::op_ret(&mut state),
            OP_HALT       => handlers::op_halt(&mut state),
            OP_DUP        => handlers::op_dup(&mut state),
            OP_DROP       => handlers::op_drop(&mut state),
            OP_SWAP       => handlers::op_swap(&mut state),
            OP_LOAD_DATA  => handlers::op_load_data(&mut state, arg),
            OP_STORE_DATA => handlers::op_store_data(&mut state, arg),
            OP_ALLOC      => handlers::op_alloc(&mut state, arg),
            OP_FREE       => handlers::op_free(&mut state),
            OP_HEAP_LOAD  => handlers::op_heap_load(&mut state),
            OP_HEAP_STORE => handlers::op_heap_store(&mut state),
            OP_SAVE       => handlers::op_save(&mut state),
            OP_RESTORE    => handlers::op_restore(&mut state),
            OP_CMP_EQ     => handlers::op_cmp_eq(&mut state),
            OP_CMP_GT     => handlers::op_cmp_gt(&mut state),
            OP_FRAME_SAVE => handlers::op_frame_save(&mut state),
            _             => Err(ExecError::InvalidOpcode(opcode)),
        }
    }

    /// Chạy liên tục: nạp lệnh → thực thi → lặp cho đến HALT hoặc lỗi
    /// (chuyển từ vm-run trong 08-dispatch.fs)
    pub fn run(&mut self) -> VmResult {
        self.running = true;
        while self.running {
            match self.fetch() {
                Ok((opcode, arg)) => {
                    if let Err(e) = self.dispatch(opcode, arg) {
                        self.running = false;
                        return VmResult::Error(e);
                    }
                }
                Err(e) => {
                    self.running = false;
                    return VmResult::Error(e);
                }
            }
        }
        VmResult::Halted
    }

    /// Chạy đúng 1 lệnh (dùng cho debugger)
    /// (chuyển từ vm-step trong 08-dispatch.fs)
    pub fn step(&mut self) -> VmResult {
        if !self.running {
            return VmResult::Halted;
        }
        match self.fetch() {
            Ok((opcode, arg)) => {
                if let Err(e) = self.dispatch(opcode, arg) {
                    self.running = false;
                    return VmResult::Error(e);
                }
                if self.running {
                    VmResult::Yielded
                } else {
                    VmResult::Halted
                }
            }
            Err(e) => {
                self.running = false;
                VmResult::Error(e)
            }
        }
    }

    /// Chạy batch N lệnh rồi yield — dùng cho async integration
    /// Cho phép Async Executor (Phase 7-9 của MyKernel) drive nhiều
    /// VM instances cùng lúc mà không block CPU.
    ///
    /// Trả về:
    ///   - Yielded: VM chưa dừng, cần gọi tiếp
    ///   - Halted:  VM dừng bình thường
    ///   - Error:   VM gặp lỗi
    pub fn run_batch(&mut self, batch_size: usize) -> VmResult {
        if !self.running {
            self.running = true;
        }
        for _ in 0..batch_size {
            if !self.running {
                return VmResult::Halted;
            }
            match self.fetch() {
                Ok((opcode, arg)) => {
                    if let Err(e) = self.dispatch(opcode, arg) {
                        self.running = false;
                        return VmResult::Error(e);
                    }
                }
                Err(e) => {
                    self.running = false;
                    return VmResult::Error(e);
                }
            }
        }
        if self.running {
            VmResult::Yielded
        } else {
            VmResult::Halted
        }
    }

    /// Tìm vị trí HALT đầu tiên trong vùng code (cho debugger)
    /// (chuyển từ find-prog-end trong 10-debugger.fs)
    pub fn find_prog_end(&mut self) {
        for i in 0..=SEG_CODE_END {
            if let Ok(cell) = self.memory.prog_read(i) {
                if unpack_opcode(cell) == OP_HALT {
                    self.prog_end = i + 1;
                    return;
                }
            }
        }
        self.prog_end = SEG_CODE_END + 1;
    }
}
