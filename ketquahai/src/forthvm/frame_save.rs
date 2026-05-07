// ============================================================
// frame_save.rs -- Frame stack cho đệ quy
// ============================================================

use crate::forthvm::value::Value;
use crate::forthvm::opcode::REG_COUNT;
use alloc::vec::Vec;

const MAX_FRAMES: usize = 64;

struct Frame {
    regs: Vec<Value>,
}

pub struct FrameStack {
    frames: Vec<Frame>,
}

impl FrameStack {
    pub fn new() -> Self {
        FrameStack { frames: Vec::new() }
    }

    pub fn save(&mut self, regs: &crate::forthvm::registers::Registers) -> bool {
        if self.frames.len() >= MAX_FRAMES { return false; }
        let mut saved = Vec::with_capacity(REG_COUNT);
        for i in 0..REG_COUNT {
            saved.push(regs.get(i).cloned().unwrap_or(Value::Nil));
        }
        self.frames.push(Frame { regs: saved });
        true
    }

    pub fn restore(&mut self, regs: &mut crate::forthvm::registers::Registers) -> bool {
        if let Some(frame) = self.frames.pop() {
            for (i, val) in frame.regs.into_iter().enumerate() {
                regs.set(i, val);
            }
            true
        } else {
            false
        }
    }

    pub fn depth(&self) -> usize { self.frames.len() }
    pub fn reset(&mut self) { self.frames.clear(); }
}
