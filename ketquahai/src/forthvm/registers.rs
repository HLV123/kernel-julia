// ============================================================
// registers.rs -- Thanh ghi R0-R7 (Value-based)
// ============================================================

use crate::forthvm::value::Value;
use crate::forthvm::opcode::REG_COUNT;
use alloc::vec::Vec;

pub struct Registers {
    regs: Vec<Value>,
}

impl Registers {
    pub fn new() -> Self {
        let mut regs = Vec::with_capacity(REG_COUNT);
        for _ in 0..REG_COUNT {
            regs.push(Value::Int(0));
        }
        Registers { regs }
    }

    pub fn get(&self, r: usize) -> Option<&Value> {
        self.regs.get(r)
    }

    pub fn set(&mut self, r: usize, val: Value) -> bool {
        if r >= REG_COUNT { return false; }
        self.regs[r] = val;
        true
    }

    pub fn reset(&mut self) {
        for r in self.regs.iter_mut() {
            *r = Value::Int(0);
        }
    }
}
