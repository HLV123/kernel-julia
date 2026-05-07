// ============================================================
// vm.rs -- Lõi VM (Stage 2: Value-based, String/Array pools)
// ============================================================

use crate::forthvm::opcode::*;
use crate::forthvm::value::{Value, StringPool, ArrayPool};
use crate::forthvm::memory::VmMemory;
use crate::forthvm::stack::DataStack;
use crate::forthvm::callstack::CallStack;
use crate::forthvm::registers::Registers;
use crate::forthvm::frame_save::FrameStack;
use crate::forthvm::syscall_bridge::SyscallBridge;
use crate::forthvm::handlers::{self, ExecError, VmState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmResult {
    Halted,
    Error(ExecError),
    Yielded,
}

pub struct ForthVm {
    pub memory: VmMemory,
    pub stack: DataStack,
    pub callstack: CallStack,
    pub registers: Registers,
    pub frames: FrameStack,
    pub bridge: SyscallBridge,
    pub strings: StringPool,
    pub arrays: ArrayPool,
    pub pc: usize,
    pub running: bool,
    pub instr_count: u64,
}

impl ForthVm {
    pub fn new() -> Self {
        ForthVm {
            memory: VmMemory::new(),
            stack: DataStack::new(),
            callstack: CallStack::new(),
            registers: Registers::new(),
            frames: FrameStack::new(),
            bridge: SyscallBridge::new(),
            strings: StringPool::new(),
            arrays: ArrayPool::new(),
            pc: 0,
            running: false,
            instr_count: 0,
        }
    }

    /// Reset toàn bộ (nhưng giữ string/array pools cho REPL)
    pub fn reset_code(&mut self) {
        self.pc = 0;
        self.stack.reset();
        self.callstack.reset();
        self.registers.reset();
        self.frames.reset();
        self.memory.reset();
        self.running = false;
        self.instr_count = 0;
    }

    /// Reset hoàn toàn (bao gồm pools)
    pub fn reset_all(&mut self) {
        self.reset_code();
        self.strings.reset();
        self.arrays.reset();
        self.memory.reset_data_only();
    }

    /// Nạp lệnh tiếp theo
    pub fn fetch(&mut self) -> Result<(u8, u32), ExecError> {
        let cell = self.memory.prog_read(self.pc)
            .map_err(|_| ExecError::MemoryOutOfBounds)?;
        self.pc += 1;
        Ok((unpack_opcode(cell), unpack_arg(cell)))
    }

    /// Phân phối lệnh
    pub fn dispatch(&mut self, opcode: u8, arg: u32) -> Result<(), ExecError> {
        let mut state = VmState {
            memory:    &mut self.memory,
            stack:     &mut self.stack,
            callstack: &mut self.callstack,
            registers: &mut self.registers,
            frames:    &mut self.frames,
            bridge:    &mut self.bridge,
            strings:   &mut self.strings,
            arrays:    &mut self.arrays,
            pc:        &mut self.pc,
            running:   &mut self.running,
        };
        handlers::dispatch(&mut state, opcode, arg)
    }

    /// Chạy liên tục đến HALT hoặc lỗi
    pub fn run(&mut self) -> VmResult {
        self.running = true;
        self.instr_count = 0;
        while self.running {
            match self.fetch() {
                Ok((opcode, arg)) => {
                    self.instr_count += 1;
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

    /// Chạy và trả giá trị đỉnh stack (cho REPL auto-print)
    pub fn run_and_get_top(&mut self) -> (VmResult, Option<Value>) {
        let result = self.run();
        let top = self.stack.pop();
        (result, top)
    }
}
