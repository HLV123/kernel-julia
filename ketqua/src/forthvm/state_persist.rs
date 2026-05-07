// ============================================================
// state_persist.rs -- Lưu/Khôi phục trạng thái VM qua VFS
// OP_SAVE: serialize VM state → ghi vào /tmp/vm_state.bin
// OP_RESTORE: đọc từ file → deserialize → load state
//
// Cải tiến so với ForthVM gốc: ghi vào file thật thay vì
// chỉ in ra màn hình
// ============================================================

use crate::forthvm::opcode::*;
use crate::forthvm::syscall_bridge::SyscallBridge;

/// Snapshot trạng thái VM — có thể serialize/deserialize
pub struct VmSnapshot {
    pub pc: u32,
    pub sp: u32,
    pub csp: u32,
    pub heap_ptr: u32,
    pub registers: [u32; REG_COUNT],
    pub stack_data: [u32; STACK_SIZE],
    pub program: [u32; PROG_SIZE],
}

/// Đường dẫn mặc định lưu trạng thái
pub const DEFAULT_SAVE_PATH: &str = "/tmp/vm_state.bin";

impl VmSnapshot {
    /// Serialize snapshot thành mảng bytes
    /// Format: header (16 bytes) + registers + stack + program
    pub fn to_bytes(&self) -> alloc::vec::Vec<u8> {
        let mut buf = alloc::vec::Vec::new();

        // Header: pc, sp, csp, heap_ptr (mỗi cái 4 bytes)
        buf.extend_from_slice(&self.pc.to_le_bytes());
        buf.extend_from_slice(&self.sp.to_le_bytes());
        buf.extend_from_slice(&self.csp.to_le_bytes());
        buf.extend_from_slice(&self.heap_ptr.to_le_bytes());

        // Registers (8 × 4 bytes = 32 bytes)
        for r in &self.registers {
            buf.extend_from_slice(&r.to_le_bytes());
        }

        // Stack data (1024 × 4 bytes)
        for s in &self.stack_data {
            buf.extend_from_slice(&s.to_le_bytes());
        }

        // Program memory (1024 × 4 bytes)
        for p in &self.program {
            buf.extend_from_slice(&p.to_le_bytes());
        }

        buf
    }

    /// Deserialize snapshot từ mảng bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        // Kiểm tra kích thước tối thiểu
        let expected_size = 16 + REG_COUNT * 4 + STACK_SIZE * 4 + PROG_SIZE * 4;
        if data.len() < expected_size {
            return None;
        }

        let mut offset = 0;

        let read_u32 = |data: &[u8], off: &mut usize| -> u32 {
            let val = u32::from_le_bytes([
                data[*off], data[*off + 1], data[*off + 2], data[*off + 3],
            ]);
            *off += 4;
            val
        };

        let pc = read_u32(data, &mut offset);
        let sp = read_u32(data, &mut offset);
        let csp = read_u32(data, &mut offset);
        let heap_ptr = read_u32(data, &mut offset);

        let mut registers = [0u32; REG_COUNT];
        for r in registers.iter_mut() {
            *r = read_u32(data, &mut offset);
        }

        let mut stack_data = [0u32; STACK_SIZE];
        for s in stack_data.iter_mut() {
            *s = read_u32(data, &mut offset);
        }

        let mut program = [0u32; PROG_SIZE];
        for p in program.iter_mut() {
            *p = read_u32(data, &mut offset);
        }

        Some(VmSnapshot {
            pc, sp, csp, heap_ptr,
            registers, stack_data, program,
        })
    }
}

/// Lưu trạng thái VM vào file (qua VFS)
pub fn save_vm_state(
    bridge: &SyscallBridge,
    snapshot: &VmSnapshot,
    path: &str,
) -> bool {
    let data = snapshot.to_bytes();
    bridge.save_to_file(path, &data)
}

/// Khôi phục trạng thái VM từ file (qua VFS)
pub fn load_vm_state(
    bridge: &SyscallBridge,
    path: &str,
) -> Option<VmSnapshot> {
    let data = bridge.load_from_file(path)?;
    VmSnapshot::from_bytes(&data)
}
