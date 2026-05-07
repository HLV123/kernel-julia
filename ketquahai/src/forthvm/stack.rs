// ============================================================
// stack.rs -- Ngăn xếp dữ liệu (Value-based)
// ============================================================

use alloc::vec::Vec;
use crate::forthvm::value::Value;
use crate::forthvm::opcode::STACK_SIZE;

pub struct DataStack {
    data: Vec<Value>,
}

impl DataStack {
    pub fn new() -> Self {
        DataStack { data: Vec::new() }
    }

    pub fn push(&mut self, val: Value) -> bool {
        if self.data.len() >= STACK_SIZE { return false; }
        self.data.push(val);
        true
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.data.last()
    }

    /// Peek N phần tử từ đỉnh (0 = đỉnh)
    pub fn peek_n(&self, n: usize) -> Option<&Value> {
        if n >= self.data.len() { return None; }
        Some(&self.data[self.data.len() - 1 - n])
    }

    pub fn depth(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    /// Swap 2 phần tử đỉnh
    pub fn swap(&mut self) -> bool {
        let len = self.data.len();
        if len < 2 { return false; }
        self.data.swap(len - 1, len - 2);
        true
    }

    /// Dup phần tử đỉnh
    pub fn dup(&mut self) -> bool {
        if let Some(top) = self.data.last().cloned() {
            self.push(top)
        } else {
            false
        }
    }

    /// Over — copy phần tử thứ 2 lên đỉnh
    pub fn over(&mut self) -> bool {
        if self.data.len() < 2 { return false; }
        let val = self.data[self.data.len() - 2].clone();
        self.push(val)
    }
}
