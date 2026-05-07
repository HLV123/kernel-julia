// ============================================================
// handlers.rs -- Bộ xử lý lệnh (Opcode Handlers)
// Mỗi lệnh máy ảo tương ứng với một hàm Rust
// (chuyển từ 07-handlers.fs sang Rust)
// ============================================================

use crate::forthvm::opcode::*;
use crate::forthvm::memory::VmMemory;
use crate::forthvm::stack::DataStack;
use crate::forthvm::callstack::CallStack;
use crate::forthvm::registers::Registers;
use crate::forthvm::frame_save::FrameStack;
use crate::forthvm::syscall_bridge::SyscallBridge;
use crate::forthvm::state_persist;

use alloc::format;

/// Lỗi khi thực thi lệnh
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecError {
    StackOverflow,
    StackUnderflow,
    CallStackOverflow,
    CallStackUnderflow,
    FrameStackOverflow,
    FrameStackUnderflow,
    InvalidRegister,
    MemoryOutOfBounds,
    HeapExhausted,
    InvalidHeapAddr,
    InvalidOpcode(u8),
    DivisionByZero,
    Halted,
}

/// Kết quả thực thi lệnh
pub type ExecResult = Result<(), ExecError>;

/// Trạng thái tham chiếu của VM — truyền vào handler
pub struct VmState<'a> {
    pub memory:    &'a mut VmMemory,
    pub stack:     &'a mut DataStack,
    pub callstack: &'a mut CallStack,
    pub registers: &'a mut Registers,
    pub frames:    &'a mut FrameStack,
    pub bridge:    &'a mut SyscallBridge,
    pub pc:        &'a mut usize,
    pub running:   &'a mut bool,
}

// --- Phase 1: Phép tính cơ bản ---

/// OP_PUSH: Đẩy hằng số lên ngăn xếp
pub fn op_push(state: &mut VmState, arg: u32) -> ExecResult {
    state.stack.push(arg).map_err(|_| ExecError::StackOverflow)
}

/// OP_ADD: Cộng 2 giá trị trên đỉnh
pub fn op_add(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    // Dùng wrapping_add cho signed arithmetic
    let result = (a as i32).wrapping_add(b as i32) as u32;
    state.stack.push(result).map_err(|_| ExecError::StackOverflow)
}

/// OP_SUB: Trừ — phần tử dưới − phần tử trên
pub fn op_sub(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let result = (a as i32).wrapping_sub(b as i32) as u32;
    state.stack.push(result).map_err(|_| ExecError::StackOverflow)
}

/// OP_MUL: Nhân 2 giá trị trên đỉnh
pub fn op_mul(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let result = (a as i32).wrapping_mul(b as i32) as u32;
    state.stack.push(result).map_err(|_| ExecError::StackOverflow)
}

/// OP_PUSH_R: Đẩy giá trị thanh ghi lên ngăn xếp
pub fn op_push_r(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.registers.get(arg as usize).map_err(|_| ExecError::InvalidRegister)?;
    state.stack.push(val).map_err(|_| ExecError::StackOverflow)
}

/// OP_POP_R: Lấy từ ngăn xếp vào thanh ghi
pub fn op_pop_r(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    state.registers.set(arg as usize, val).map_err(|_| ExecError::InvalidRegister)
}

/// OP_PRINT: In và xoá giá trị trên đỉnh (qua syscall bridge)
pub fn op_print(state: &mut VmState) -> ExecResult {
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    // Giá trị được in như signed i32 (giống Forth `. `)
    state.bridge.print_value(val as i32);
    Ok(())
}

/// OP_HALT: Dừng máy ảo
pub fn op_halt(state: &mut VmState) -> ExecResult {
    *state.running = false;
    Ok(())
}

/// OP_DUP: Nhân đôi giá trị trên đỉnh
pub fn op_dup(state: &mut VmState) -> ExecResult {
    let val = state.stack.peek().map_err(|_| ExecError::StackUnderflow)?;
    state.stack.push(val).map_err(|_| ExecError::StackOverflow)
}

/// OP_DROP: Xoá giá trị trên đỉnh
pub fn op_drop(state: &mut VmState) -> ExecResult {
    state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    Ok(())
}

/// OP_SWAP: Đổi chỗ 2 giá trị trên đỉnh
pub fn op_swap(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    state.stack.push(b).map_err(|_| ExecError::StackOverflow)?;
    state.stack.push(a).map_err(|_| ExecError::StackOverflow)
}

// --- Phase 2: Điều khiển luồng ---

/// OP_JMP: Nhảy vô điều kiện
pub fn op_jmp(state: &mut VmState, arg: u32) -> ExecResult {
    *state.pc = arg as usize;
    Ok(())
}

/// OP_JZ: Nhảy nếu giá trị = 0
pub fn op_jz(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    if val == 0 {
        *state.pc = arg as usize;
    }
    Ok(())
}

/// OP_JGT: Nhảy nếu > 0
pub fn op_jgt(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    if (val as i32) > 0 {
        *state.pc = arg as usize;
    }
    Ok(())
}

// --- Phase 3: Gọi hàm ---

/// OP_CALL: Gọi chương trình con — lưu PC vào call stack, nhảy tới arg
pub fn op_call(state: &mut VmState, arg: u32) -> ExecResult {
    let return_addr = *state.pc;
    state.callstack.push(return_addr).map_err(|_| ExecError::CallStackOverflow)?;
    *state.pc = arg as usize;
    Ok(())
}

/// OP_RET: Trở về từ chương trình con — khôi phục PC từ call stack
pub fn op_ret(state: &mut VmState) -> ExecResult {
    let addr = state.callstack.pop().map_err(|_| ExecError::CallStackUnderflow)?;
    *state.pc = addr;
    Ok(())
}

// --- Phase 5: Bộ nhớ ---

/// OP_LOAD_DATA: Đọc biến toàn cục từ vùng Data
pub fn op_load_data(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.memory.data_read(arg as usize).map_err(|_| ExecError::MemoryOutOfBounds)?;
    state.stack.push(val).map_err(|_| ExecError::StackOverflow)
}

/// OP_STORE_DATA: Ghi biến toàn cục vào vùng Data
pub fn op_store_data(state: &mut VmState, arg: u32) -> ExecResult {
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    state.memory.data_write(arg as usize, val).map_err(|_| ExecError::MemoryOutOfBounds)
}

/// OP_ALLOC: Cấp phát bộ nhớ heap
pub fn op_alloc(state: &mut VmState, arg: u32) -> ExecResult {
    let addr = state.memory.heap_alloc(arg as usize).map_err(|_| ExecError::HeapExhausted)?;
    state.stack.push(addr as u32).map_err(|_| ExecError::StackOverflow)
}

/// OP_FREE: Giải phóng bộ nhớ heap
pub fn op_free(state: &mut VmState) -> ExecResult {
    let addr = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    state.memory.heap_free(addr as usize).map_err(|_| ExecError::InvalidHeapAddr)
}

/// OP_HEAP_LOAD: Đọc từ heap (địa chỉ trên đỉnh stack)
pub fn op_heap_load(state: &mut VmState) -> ExecResult {
    let addr = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let val = state.memory.prog_read(addr as usize).map_err(|_| ExecError::MemoryOutOfBounds)?;
    state.stack.push(val).map_err(|_| ExecError::StackOverflow)
}

/// OP_HEAP_STORE: Ghi vào heap (giá trị và địa chỉ trên stack)
pub fn op_heap_store(state: &mut VmState) -> ExecResult {
    let addr = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let val = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    state.memory.prog_write(addr as usize, val).map_err(|_| ExecError::MemoryOutOfBounds)
}

// --- Phase 7: Trạng thái ---

/// OP_SAVE: Lưu trạng thái VM ra file qua VFS
/// Cải tiến: ghi vào file thật thay vì chỉ in
pub fn op_save(state: &mut VmState) -> ExecResult {
    // Hiển thị trạng thái ra serial (tương thích với Forth gốc)
    state.bridge.println_str("=== TRANG THAI VM ===");
    state.bridge.print_str(&format!(
        "PC: {}  SP: {}  CSP: {}  HEAP: {}\n",
        *state.pc, state.stack.sp, state.callstack.csp, state.memory.heap_ptr
    ));

    // In thanh ghi
    state.bridge.print_str("Thanh ghi:");
    for i in 0..REG_COUNT {
        if let Ok(val) = state.registers.get(i) {
            state.bridge.print_str(&format!("  R{}={}", i, val as i32));
        }
    }
    state.bridge.print_str("\n");

    // In ngăn xếp
    state.bridge.print_str(&format!("Ngan xep ({} phan tu):\n", state.stack.sp));
    for i in 0..state.stack.sp {
        if let Some(val) = state.stack.get(i) {
            state.bridge.print_str(&format!("  {}: {}\n", i, val as i32));
        }
    }
    state.bridge.println_str("=== KET THUC ===");

    // Lưu vào file VFS
    let mut stack_data = [0u32; STACK_SIZE];
    for i in 0..state.stack.sp {
        if let Some(val) = state.stack.get(i) {
            stack_data[i] = val;
        }
    }

    let snapshot = state_persist::VmSnapshot {
        pc: *state.pc as u32,
        sp: state.stack.sp as u32,
        csp: state.callstack.csp as u32,
        heap_ptr: state.memory.heap_ptr as u32,
        registers: *state.registers.as_array(),
        stack_data,
        program: state.memory.program,
    };

    state_persist::save_vm_state(state.bridge, &snapshot, state_persist::DEFAULT_SAVE_PATH);

    Ok(())
}

/// OP_RESTORE: Khôi phục trạng thái VM từ file
pub fn op_restore(state: &mut VmState) -> ExecResult {
    match state_persist::load_vm_state(state.bridge, state_persist::DEFAULT_SAVE_PATH) {
        Some(snapshot) => {
            *state.pc = snapshot.pc as usize;
            state.stack.reset();
            for i in 0..(snapshot.sp as usize) {
                let _ = state.stack.push(snapshot.stack_data[i]);
            }
            state.callstack.reset();
            state.memory.heap_ptr = snapshot.heap_ptr as usize;
            for i in 0..REG_COUNT {
                let _ = state.registers.set(i, snapshot.registers[i]);
            }
            state.memory.program = snapshot.program;
            state.bridge.println_str("Khoi phuc trang thai thanh cong.");
        }
        None => {
            state.bridge.println_str("Khoi phuc tu file that bai.");
        }
    }
    Ok(())
}

// --- Phase 8: So sánh ---

/// OP_CMP_EQ: So sánh bằng — đẩy 1 nếu bằng, 0 nếu khác
pub fn op_cmp_eq(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let result = if a == b { 1u32 } else { 0u32 };
    state.stack.push(result).map_err(|_| ExecError::StackOverflow)
}

/// OP_CMP_GT: So sánh lớn hơn — đẩy 1 nếu a > b (signed)
pub fn op_cmp_gt(state: &mut VmState) -> ExecResult {
    let b = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let a = state.stack.pop().map_err(|_| ExecError::StackUnderflow)?;
    let result = if (a as i32) > (b as i32) { 1u32 } else { 0u32 };
    state.stack.push(result).map_err(|_| ExecError::StackOverflow)
}

// --- Phase 9: Frame Save ---

/// OP_FRAME_SAVE: Lưu register frame cho đệ quy đôi
pub fn op_frame_save(state: &mut VmState) -> ExecResult {
    state.frames.save(
        *state.pc,
        state.registers.as_array(),
        state.stack.sp,
    ).map_err(|_| ExecError::FrameStackOverflow)
}
