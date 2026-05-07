// ============================================================
// callstack.rs -- Ngăn xếp lời gọi hàm
// Lưu địa chỉ trả về khi CALL, khôi phục khi RET
// (chuyển từ 05-callstack.fs sang Rust)
// ============================================================

use crate::forthvm::opcode::CSTACK_SIZE;

/// Lỗi ngăn xếp gọi
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CStackError {
    Overflow,
    Underflow,
}

/// Ngăn xếp lời gọi hàm – lưu địa chỉ trả về
pub struct CallStack {
    /// Dữ liệu ngăn xếp gọi
    data: [usize; CSTACK_SIZE],
    /// Con trỏ ngăn xếp gọi (call stack pointer)
    pub csp: usize,
}

impl CallStack {
    /// Tạo ngăn xếp gọi mới, rỗng
    pub fn new() -> Self {
        CallStack {
            data: [0usize; CSTACK_SIZE],
            csp: 0,
        }
    }

    /// Đẩy địa chỉ trả về lên ngăn xếp gọi
    pub fn push(&mut self, addr: usize) -> Result<(), CStackError> {
        if self.csp >= CSTACK_SIZE {
            return Err(CStackError::Overflow);
        }
        self.data[self.csp] = addr;
        self.csp += 1;
        Ok(())
    }

    /// Lấy địa chỉ trả về từ ngăn xếp gọi
    pub fn pop(&mut self) -> Result<usize, CStackError> {
        if self.csp == 0 {
            return Err(CStackError::Underflow);
        }
        self.csp -= 1;
        Ok(self.data[self.csp])
    }

    /// Reset ngăn xếp gọi về rỗng
    pub fn reset(&mut self) {
        self.csp = 0;
    }

    /// Số phần tử hiện tại
    pub fn len(&self) -> usize {
        self.csp
    }

    /// Ngăn xếp gọi có rỗng không
    pub fn is_empty(&self) -> bool {
        self.csp == 0
    }
}
