// ============================================================
// opcode.rs -- Hằng số hệ thống
// Định nghĩa kích thước bộ nhớ, mã lệnh, phân vùng
// (chuyển từ 00-constants.fs sang Rust)
// ============================================================

// --- Kích thước bộ nhớ ---
pub const PROG_SIZE:   usize = 1024;  // Vùng chương trình (cells)
pub const STACK_SIZE:  usize = 1024;  // Ngăn xếp dữ liệu (cells)
pub const CSTACK_SIZE: usize = 256;   // Ngăn xếp lời gọi hàm (cells)
pub const REG_COUNT:   usize = 8;     // Số thanh ghi R0–R7

// --- Phân vùng bộ nhớ chương trình ---
// Mảng program[] được chia thành 4 vùng:
//   [0..255]    Code   – chứa bytecode
//   [256..383]  Data   – biến toàn cục
//   [384..511]  Stack  – dự phòng
//   [512..1023] Heap   – bộ nhớ động (arena)
pub const SEG_CODE_BASE:  usize = 0;
pub const SEG_DATA_BASE:  usize = 256;
pub const SEG_STACK_BASE: usize = 384;
pub const SEG_HEAP_BASE:  usize = 512;
pub const SEG_CODE_END:   usize = 255;
pub const SEG_DATA_END:   usize = 383;
pub const SEG_STACK_END:  usize = 511;
pub const SEG_HEAP_END:   usize = 1023;

// --- Mã lệnh (Opcode) ---
// Mỗi lệnh mã hoá thành 1 cell: (arg << 8) | opcode
// 8 bit thấp = mã lệnh, các bit còn lại = tham số
pub const OP_PUSH:       u8 = 0;   // Đẩy giá trị lên ngăn xếp
pub const OP_ADD:        u8 = 1;   // Cộng 2 giá trị trên đỉnh
pub const OP_SUB:        u8 = 2;   // Trừ: phần tử dưới − phần tử trên
pub const OP_MUL:        u8 = 3;   // Nhân 2 giá trị trên đỉnh
pub const OP_PUSH_R:     u8 = 4;   // Đẩy giá trị thanh ghi lên ngăn xếp
pub const OP_POP_R:      u8 = 5;   // Lấy từ ngăn xếp vào thanh ghi
pub const OP_PRINT:      u8 = 6;   // In và xoá giá trị trên đỉnh
pub const OP_JMP:        u8 = 7;   // Nhảy vô điều kiện
pub const OP_JZ:         u8 = 8;   // Nhảy nếu giá trị = 0
pub const OP_CALL:       u8 = 9;   // Gọi chương trình con
pub const OP_RET:        u8 = 10;  // Trở về từ chương trình con
pub const OP_HALT:       u8 = 11;  // Dừng máy ảo
pub const OP_DUP:        u8 = 12;  // Nhân đôi giá trị trên đỉnh
pub const OP_DROP:       u8 = 13;  // Xoá giá trị trên đỉnh
pub const OP_SWAP:       u8 = 14;  // Đổi chỗ 2 giá trị trên đỉnh
pub const OP_LOAD_DATA:  u8 = 15;  // Đọc từ vùng dữ liệu
pub const OP_STORE_DATA: u8 = 16;  // Ghi vào vùng dữ liệu
pub const OP_ALLOC:      u8 = 17;  // Cấp phát bộ nhớ heap
pub const OP_FREE:       u8 = 18;  // Giải phóng bộ nhớ heap
pub const OP_HEAP_LOAD:  u8 = 19;  // Đọc từ heap
pub const OP_HEAP_STORE: u8 = 20;  // Ghi vào heap
pub const OP_JGT:        u8 = 21;  // Nhảy nếu > 0
pub const OP_SAVE:       u8 = 22;  // Lưu trạng thái VM
pub const OP_RESTORE:    u8 = 23;  // Khôi phục trạng thái VM
pub const OP_CMP_EQ:     u8 = 24;  // So sánh bằng: đẩy 1 nếu bằng
pub const OP_CMP_GT:     u8 = 25;  // So sánh lớn hơn: đẩy 1 nếu >
pub const OP_FRAME_SAVE: u8 = 26;  // Lưu frame cho đệ quy đôi (Phase 9)

// --- Hàm trợ giúp ---

/// Đóng gói tham số và mã lệnh thành 1 cell
/// Format: (arg << 8) | opcode
#[inline]
pub fn pack(arg: u32, opcode: u8) -> u32 {
    (arg << 8) | (opcode as u32)
}

/// Tách mã lệnh từ cell đã đóng gói
#[inline]
pub fn unpack_opcode(cell: u32) -> u8 {
    (cell & 0xFF) as u8
}

/// Tách tham số từ cell đã đóng gói
#[inline]
pub fn unpack_arg(cell: u32) -> u32 {
    cell >> 8
}

/// Tên opcodes (dùng cho disassembler và debugger)
pub fn opcode_name(op: u8) -> &'static str {
    match op {
        OP_PUSH       => "PUSH",
        OP_ADD        => "ADD",
        OP_SUB        => "SUB",
        OP_MUL        => "MUL",
        OP_PUSH_R     => "PUSH_R",
        OP_POP_R      => "POP_R",
        OP_PRINT      => "PRINT",
        OP_JMP        => "JMP",
        OP_JZ         => "JZ",
        OP_CALL       => "CALL",
        OP_RET        => "RET",
        OP_HALT       => "HALT",
        OP_DUP        => "DUP",
        OP_DROP       => "DROP",
        OP_SWAP       => "SWAP",
        OP_LOAD_DATA  => "LOAD_DATA",
        OP_STORE_DATA => "STORE_DATA",
        OP_ALLOC      => "ALLOC",
        OP_FREE       => "FREE",
        OP_HEAP_LOAD  => "HEAP_LOAD",
        OP_HEAP_STORE => "HEAP_STORE",
        OP_JGT        => "JGT",
        OP_SAVE       => "SAVE",
        OP_RESTORE    => "RESTORE",
        OP_CMP_EQ     => "CMP_EQ",
        OP_CMP_GT     => "CMP_GT",
        OP_FRAME_SAVE => "FRAME_SAVE",
        _             => "???",
    }
}

/// Opcode có tham số hay không (dùng cho assembler/disassembler)
pub fn opcode_has_arg(op: u8) -> bool {
    matches!(op,
        OP_PUSH | OP_PUSH_R | OP_POP_R |
        OP_JMP | OP_JZ | OP_JGT | OP_CALL |
        OP_LOAD_DATA | OP_STORE_DATA | OP_ALLOC |
        OP_SAVE | OP_RESTORE | OP_FRAME_SAVE
    )
}
