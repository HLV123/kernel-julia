// ============================================================
// frame_save.rs -- Lưu frame cho đệ quy đôi (Phase 9 Planned)
// Mỗi frame lưu: địa chỉ trả về + registers + SP marker
// Cho phép mutual recursion: f() gọi g() gọi f() ...
// ============================================================

use crate::forthvm::opcode::REG_COUNT;

/// Kích thước tối đa ngăn xếp frame
pub const FRAME_STACK_SIZE: usize = 64;

/// Lỗi frame stack
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    Overflow,
    Underflow,
}

/// Một stack frame — lưu trạng thái khi gọi hàm
#[derive(Clone, Copy)]
pub struct StackFrame {
    /// Địa chỉ trả về (PC khi gọi CALL)
    pub return_addr: usize,
    /// Bản sao thanh ghi tại thời điểm gọi
    pub saved_registers: [u32; REG_COUNT],
    /// Stack pointer tại thời điểm gọi (để khôi phục stack)
    pub saved_sp: usize,
}

impl Default for StackFrame {
    fn default() -> Self {
        StackFrame {
            return_addr: 0,
            saved_registers: [0u32; REG_COUNT],
            saved_sp: 0,
        }
    }
}

/// Ngăn xếp frame — hỗ trợ đệ quy đôi hoàn chỉnh
///
/// Khi CALL: push frame (lưu PC + registers + SP)
/// Khi RET:  pop frame (khôi phục registers + SP, jump tới saved PC)
///
/// Điều này cho phép:
///   - fact(5) → fact(4) → fact(3) → ... (đệ quy đơn)
///   - isEven(5) → isOdd(4) → isEven(3) → ... (đệ quy đôi)
///   - Mỗi hàm có register context riêng, không bị ghi đè
pub struct FrameStack {
    frames: [StackFrame; FRAME_STACK_SIZE],
    /// Frame pointer — chỉ số frame hiện tại
    pub fp: usize,
}

impl FrameStack {
    pub fn new() -> Self {
        FrameStack {
            frames: [StackFrame::default(); FRAME_STACK_SIZE],
            fp: 0,
        }
    }

    /// Lưu frame mới (khi CALL hoặc OP_FRAME_SAVE)
    pub fn save(
        &mut self,
        return_addr: usize,
        registers: &[u32; REG_COUNT],
        sp: usize,
    ) -> Result<(), FrameError> {
        if self.fp >= FRAME_STACK_SIZE {
            return Err(FrameError::Overflow);
        }
        self.frames[self.fp] = StackFrame {
            return_addr,
            saved_registers: *registers,
            saved_sp: sp,
        };
        self.fp += 1;
        Ok(())
    }

    /// Khôi phục frame (khi RET)
    /// Trả về (return_addr, saved_registers, saved_sp)
    pub fn restore(&mut self) -> Result<StackFrame, FrameError> {
        if self.fp == 0 {
            return Err(FrameError::Underflow);
        }
        self.fp -= 1;
        Ok(self.frames[self.fp])
    }

    /// Reset frame stack
    pub fn reset(&mut self) {
        self.fp = 0;
    }

    /// Số frame hiện tại
    pub fn depth(&self) -> usize {
        self.fp
    }
}
