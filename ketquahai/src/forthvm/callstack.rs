// ============================================================
// callstack.rs -- Ngăn xếp gọi hàm (giữ nguyên u32)
// ============================================================

use crate::forthvm::opcode::CSTACK_SIZE;

pub struct CallStack {
    data: [usize; CSTACK_SIZE],
    sp: usize,
}

impl CallStack {
    pub fn new() -> Self {
        CallStack { data: [0; CSTACK_SIZE], sp: 0 }
    }

    pub fn push(&mut self, addr: usize) -> bool {
        if self.sp >= CSTACK_SIZE { return false; }
        self.data[self.sp] = addr;
        self.sp += 1;
        true
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.sp == 0 { return None; }
        self.sp -= 1;
        Some(self.data[self.sp])
    }

    pub fn depth(&self) -> usize { self.sp }
    pub fn reset(&mut self) { self.sp = 0; }
}
