// ============================================================
// stack.rs -- Ngăn xếp máy ảo
// Quy ước Empty-Ascending: sp trỏ vào ô tiếp theo sẽ ghi
//   PUSH: ghi vào stack[sp], rồi tăng sp
//   POP:  giảm sp, rồi đọc từ stack[sp]
// (chuyển từ 02-stack.fs sang Rust)
// ============================================================

use crate::forthvm::opcode::STACK_SIZE;

/// Lỗi ngăn xếp
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackError {
    Overflow,
    Underflow,
}

/// Ngăn xếp dữ liệu của máy ảo
pub struct DataStack {
    /// Dữ liệu ngăn xếp
    data: [u32; STACK_SIZE],
    /// Con trỏ đỉnh ngăn xếp (stack pointer)
    pub sp: usize,
}

impl DataStack {
    /// Tạo ngăn xếp mới, rỗng
    pub fn new() -> Self {
        DataStack {
            data: [0u32; STACK_SIZE],
            sp: 0,
        }
    }

    /// Đẩy giá trị lên đỉnh ngăn xếp
    pub fn push(&mut self, val: u32) -> Result<(), StackError> {
        if self.sp >= STACK_SIZE {
            return Err(StackError::Overflow);
        }
        self.data[self.sp] = val;
        self.sp += 1;
        Ok(())
    }

    /// Lấy giá trị từ đỉnh ngăn xếp
    pub fn pop(&mut self) -> Result<u32, StackError> {
        if self.sp == 0 {
            return Err(StackError::Underflow);
        }
        self.sp -= 1;
        Ok(self.data[self.sp])
    }

    /// Xem giá trị trên đỉnh mà không lấy ra
    pub fn peek(&self) -> Result<u32, StackError> {
        if self.sp == 0 {
            return Err(StackError::Underflow);
        }
        Ok(self.data[self.sp - 1])
    }

    /// Đọc giá trị tại vị trí index (từ đáy ngăn xếp)
    pub fn get(&self, index: usize) -> Option<u32> {
        if index < self.sp {
            Some(self.data[index])
        } else {
            None
        }
    }

    /// Reset ngăn xếp về rỗng
    pub fn reset(&mut self) {
        self.sp = 0;
    }

    /// Số phần tử hiện tại
    pub fn len(&self) -> usize {
        self.sp
    }

    /// Ngăn xếp có rỗng không
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }
}
