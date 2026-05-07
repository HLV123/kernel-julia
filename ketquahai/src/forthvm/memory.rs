// ============================================================
// memory.rs -- Bộ nhớ chương trình (bytecode) + Data slots
// ============================================================

use crate::forthvm::opcode::{PROG_SIZE, DATA_SLOTS};
use crate::forthvm::value::Value;
use alloc::vec::Vec;

pub struct VmMemory {
    /// Vùng bytecode (vẫn là u32 vì bytecode = packed opcode+arg)
    prog: [u32; PROG_SIZE],
    /// Vùng dữ liệu (biến) — giờ là Value
    data: Vec<Value>,
}

impl VmMemory {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(DATA_SLOTS);
        for _ in 0..DATA_SLOTS {
            data.push(Value::Nil);
        }
        VmMemory {
            prog: [0u32; PROG_SIZE],
            data,
        }
    }

    // --- Bytecode ---

    pub fn prog_read(&self, addr: usize) -> Result<u32, ()> {
        if addr >= PROG_SIZE { return Err(()); }
        Ok(self.prog[addr])
    }

    pub fn prog_write(&mut self, addr: usize, val: u32) -> Result<(), ()> {
        if addr >= PROG_SIZE { return Err(()); }
        self.prog[addr] = val;
        Ok(())
    }

    // --- Data slots ---

    pub fn data_load(&self, slot: usize) -> Option<Value> {
        self.data.get(slot).cloned()
    }

    pub fn data_store(&mut self, slot: usize, val: Value) -> bool {
        if slot >= DATA_SLOTS { return false; }
        // Tự mở rộng nếu cần
        while self.data.len() <= slot {
            self.data.push(Value::Nil);
        }
        self.data[slot] = val;
        true
    }

    pub fn reset(&mut self) {
        self.prog = [0u32; PROG_SIZE];
        for d in self.data.iter_mut() {
            *d = Value::Nil;
        }
    }

    pub fn reset_data_only(&mut self) {
        for d in self.data.iter_mut() {
            *d = Value::Nil;
        }
    }
}
