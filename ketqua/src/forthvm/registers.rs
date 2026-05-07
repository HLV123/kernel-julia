// ============================================================
// registers.rs -- Thanh ghi R0–R7
// 8 thanh ghi đa dụng, truy cập qua chỉ số 0–7
// (chuyển từ 03-registers.fs sang Rust)
// ============================================================

use crate::forthvm::opcode::REG_COUNT;

/// Lỗi thanh ghi
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegError {
    InvalidIndex,
}

/// 8 thanh ghi đa dụng R0–R7
pub struct Registers {
    data: [u32; REG_COUNT],
}

impl Registers {
    /// Tạo thanh ghi mới, tất cả = 0
    pub fn new() -> Self {
        Registers {
            data: [0u32; REG_COUNT],
        }
    }

    /// Đọc giá trị thanh ghi (reg → val)
    pub fn get(&self, index: usize) -> Result<u32, RegError> {
        if index >= REG_COUNT {
            return Err(RegError::InvalidIndex);
        }
        Ok(self.data[index])
    }

    /// Ghi giá trị vào thanh ghi (val → reg)
    pub fn set(&mut self, index: usize, val: u32) -> Result<(), RegError> {
        if index >= REG_COUNT {
            return Err(RegError::InvalidIndex);
        }
        self.data[index] = val;
        Ok(())
    }

    /// Lấy tham chiếu tới mảng thanh ghi (dùng cho snapshot)
    pub fn as_array(&self) -> &[u32; REG_COUNT] {
        &self.data
    }

    /// Reset tất cả thanh ghi về 0
    pub fn reset(&mut self) {
        self.data = [0u32; REG_COUNT];
    }
}
