// ============================================================
// memory.rs -- Bố trí bộ nhớ máy ảo
// Tạo 4 vùng nhớ chính và các biến trạng thái
// (chuyển từ 01-memory.fs sang Rust)
// ============================================================
//
// Trong ForthVM gốc, program[] là mảng tĩnh 1024 cells chia 4 phân vùng:
//   [0..255]    Code   – bytecode
//   [256..383]  Data   – biến toàn cục
//   [384..511]  Stack  – dự phòng
//   [512..1023] Heap   – bộ nhớ động arena
//
// Trong Rust, chúng ta dùng owned array trong struct — mỗi VM instance
// sở hữu memory riêng, an toàn cho multi-instance.
//
// Khi tích hợp vào Ring 3 process, có thể thay bằng raw pointer tới
// vùng mmap() trong Virtual Address Space (Phase 12), tận dụng SMAP/SMEP
// từ Phase 24.

use crate::forthvm::opcode::*;

/// Lỗi truy cập bộ nhớ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemError {
    OutOfBounds,
    HeapExhausted,
    InvalidHeapAddr,
}

/// Bộ nhớ chính của máy ảo
/// Chứa bytecode + dữ liệu + heap trong một mảng liên tục
pub struct VmMemory {
    /// Mảng chương trình: bytecode + data + stack segment + heap
    pub program: [u32; PROG_SIZE],
    /// Con trỏ heap – vị trí tự do tiếp theo
    pub heap_ptr: usize,
}

impl VmMemory {
    /// Tạo bộ nhớ mới, toàn bộ được zero-init
    pub fn new() -> Self {
        VmMemory {
            program: [0u32; PROG_SIZE],
            heap_ptr: SEG_HEAP_BASE,
        }
    }

    /// Khởi tạo lại heap
    pub fn heap_init(&mut self) {
        self.heap_ptr = SEG_HEAP_BASE;
    }

    /// Đọc giá trị tại vị trí addr trong program[]
    pub fn prog_read(&self, addr: usize) -> Result<u32, MemError> {
        if addr >= PROG_SIZE {
            return Err(MemError::OutOfBounds);
        }
        Ok(self.program[addr])
    }

    /// Ghi giá trị vào vị trí addr trong program[]
    pub fn prog_write(&mut self, addr: usize, val: u32) -> Result<(), MemError> {
        if addr >= PROG_SIZE {
            return Err(MemError::OutOfBounds);
        }
        self.program[addr] = val;
        Ok(())
    }

    // --- Vùng Data: biến toàn cục ---
    // Slot 0 → program[256], slot 1 → program[257], ...

    /// Đọc biến toàn cục tại slot
    pub fn data_read(&self, slot: usize) -> Result<u32, MemError> {
        let addr = SEG_DATA_BASE + slot;
        if addr > SEG_DATA_END {
            return Err(MemError::OutOfBounds);
        }
        Ok(self.program[addr])
    }

    /// Ghi biến toàn cục tại slot
    pub fn data_write(&mut self, slot: usize, val: u32) -> Result<(), MemError> {
        let addr = SEG_DATA_BASE + slot;
        if addr > SEG_DATA_END {
            return Err(MemError::OutOfBounds);
        }
        self.program[addr] = val;
        Ok(())
    }

    // --- Vùng Heap: cấp phát động kiểu arena ---

    /// Cấp phát n cells liên tiếp trên heap, trả về địa chỉ đầu
    pub fn heap_alloc(&mut self, n: usize) -> Result<usize, MemError> {
        if self.heap_ptr + n > SEG_HEAP_END + 1 {
            return Err(MemError::HeapExhausted);
        }
        let addr = self.heap_ptr;
        self.heap_ptr += n;
        Ok(addr)
    }

    /// Giải phóng – đặt lại con trỏ heap về addr
    /// (chỉ an toàn nếu giải phóng theo thứ tự ngược)
    pub fn heap_free(&mut self, addr: usize) -> Result<(), MemError> {
        if addr < SEG_HEAP_BASE {
            return Err(MemError::InvalidHeapAddr);
        }
        self.heap_ptr = addr;
        Ok(())
    }

    /// Xoá toàn bộ bộ nhớ và reset heap
    pub fn reset(&mut self) {
        self.program = [0u32; PROG_SIZE];
        self.heap_ptr = SEG_HEAP_BASE;
    }
}
