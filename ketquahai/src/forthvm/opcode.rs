// ============================================================
// opcode.rs -- Hằng số hệ thống (Stage 2: mở rộng)
// ~55 opcodes hỗ trợ Int, Bool, String, Array
// ============================================================

pub const PROG_SIZE: usize = 4096;  // Mở rộng x4 cho code phức tạp
pub const STACK_SIZE: usize = 1024;
pub const CSTACK_SIZE: usize = 256;
pub const REG_COUNT: usize = 8;
pub const DATA_SLOTS: usize = 256;

// --- Mã lệnh (Opcode) ---

// Ngăn xếp & hằng số
pub const OP_PUSH_INT:    u8 = 0;
pub const OP_PUSH_TRUE:   u8 = 1;
pub const OP_PUSH_FALSE:  u8 = 2;
pub const OP_PUSH_NIL:    u8 = 3;
pub const OP_PUSH_STR:    u8 = 4;   // arg = string pool index
pub const OP_DUP:         u8 = 5;
pub const OP_DROP:        u8 = 6;
pub const OP_SWAP:        u8 = 7;

// Số học
pub const OP_ADD:         u8 = 10;
pub const OP_SUB:         u8 = 11;
pub const OP_MUL:         u8 = 12;
pub const OP_DIV:         u8 = 13;
pub const OP_MOD:         u8 = 14;
pub const OP_POW:         u8 = 15;
pub const OP_NEG:         u8 = 16;  // Đảo dấu

// So sánh
pub const OP_CMP_EQ:      u8 = 20;
pub const OP_CMP_NEQ:     u8 = 21;
pub const OP_CMP_LT:      u8 = 22;
pub const OP_CMP_GT:      u8 = 23;
pub const OP_CMP_LTE:     u8 = 24;
pub const OP_CMP_GTE:     u8 = 25;

// Logic
pub const OP_AND:         u8 = 30;
pub const OP_OR:          u8 = 31;
pub const OP_NOT:         u8 = 32;

// Bitwise
pub const OP_BAND:        u8 = 33;
pub const OP_BOR:         u8 = 34;
pub const OP_BXOR:        u8 = 35;
pub const OP_SHL:         u8 = 36;
pub const OP_SHR:         u8 = 37;

// Điều khiển luồng
pub const OP_JMP:         u8 = 40;
pub const OP_JZ:          u8 = 41;  // Jump if falsy
pub const OP_JNZ:         u8 = 42;  // Jump if truthy
pub const OP_CALL:        u8 = 43;
pub const OP_RET:         u8 = 44;
pub const OP_HALT:        u8 = 45;

// Dữ liệu
pub const OP_LOAD:        u8 = 50;  // Load biến (slot index)
pub const OP_STORE:       u8 = 51;  // Store biến (slot index)
pub const OP_PUSH_R:      u8 = 52;  // Push register
pub const OP_POP_R:       u8 = 53;  // Pop to register

// I/O
pub const OP_PRINT:       u8 = 60;  // println — in + xuống dòng
pub const OP_PRINT_NOLF:  u8 = 61;  // print — in không xuống dòng
pub const OP_READLINE:    u8 = 62;  // Đọc dòng từ input

// Chuỗi
pub const OP_STR_CONCAT:  u8 = 70;  // Ghép chuỗi
pub const OP_STR_LEN:     u8 = 71;  // Độ dài chuỗi
pub const OP_STR_INTERP:  u8 = 72;  // Nối chuỗi + giá trị (cho interpolation)
pub const OP_TO_STRING:   u8 = 73;  // Chuyển giá trị → chuỗi

// Mảng
pub const OP_ARR_NEW:     u8 = 80;  // Tạo mảng rỗng
pub const OP_ARR_PUSH:    u8 = 81;  // push!(arr, val)
pub const OP_ARR_POP:     u8 = 82;  // pop!(arr)
pub const OP_ARR_GET:     u8 = 83;  // arr[index]
pub const OP_ARR_SET:     u8 = 84;  // arr[index] = val
pub const OP_ARR_LEN:     u8 = 85;  // length(arr)
pub const OP_ARR_LITERAL: u8 = 86;  // arg = count, tạo mảng từ stack

// Built-in functions
pub const OP_BUILTIN:     u8 = 90;  // arg = builtin function id

// Trạng thái
pub const OP_SAVE:        u8 = 95;
pub const OP_RESTORE:     u8 = 96;

// --- Hàm trợ giúp ---

#[inline]
pub fn pack(arg: u32, opcode: u8) -> u32 {
    (arg << 8) | (opcode as u32)
}

#[inline]
pub fn unpack_opcode(cell: u32) -> u8 {
    (cell & 0xFF) as u8
}

#[inline]
pub fn unpack_arg(cell: u32) -> u32 {
    cell >> 8
}

/// Tách arg thành signed i32 (24-bit sign-extended)
#[inline]
pub fn unpack_arg_signed(cell: u32) -> i32 {
    let raw = cell >> 8;
    // Sign extend from 24 bits
    if raw & 0x800000 != 0 {
        (raw | 0xFF000000) as i32
    } else {
        raw as i32
    }
}

/// Đóng gói signed arg
#[inline]
pub fn pack_signed(arg: i32, opcode: u8) -> u32 {
    let arg_bits = (arg as u32) & 0xFFFFFF;
    (arg_bits << 8) | (opcode as u32)
}

pub fn opcode_name(op: u8) -> &'static str {
    match op {
        OP_PUSH_INT    => "PUSH_INT",
        OP_PUSH_TRUE   => "PUSH_TRUE",
        OP_PUSH_FALSE  => "PUSH_FALSE",
        OP_PUSH_NIL    => "PUSH_NIL",
        OP_PUSH_STR    => "PUSH_STR",
        OP_DUP         => "DUP",
        OP_DROP        => "DROP",
        OP_SWAP        => "SWAP",
        OP_ADD         => "ADD",
        OP_SUB         => "SUB",
        OP_MUL         => "MUL",
        OP_DIV         => "DIV",
        OP_MOD         => "MOD",
        OP_POW         => "POW",
        OP_NEG         => "NEG",
        OP_CMP_EQ      => "CMP_EQ",
        OP_CMP_NEQ     => "CMP_NEQ",
        OP_CMP_LT      => "CMP_LT",
        OP_CMP_GT      => "CMP_GT",
        OP_CMP_LTE     => "CMP_LTE",
        OP_CMP_GTE     => "CMP_GTE",
        OP_AND         => "AND",
        OP_OR          => "OR",
        OP_NOT         => "NOT",
        OP_BAND        => "BAND",
        OP_BOR         => "BOR",
        OP_BXOR        => "BXOR",
        OP_SHL         => "SHL",
        OP_SHR         => "SHR",
        OP_JMP         => "JMP",
        OP_JZ          => "JZ",
        OP_JNZ         => "JNZ",
        OP_CALL        => "CALL",
        OP_RET         => "RET",
        OP_HALT        => "HALT",
        OP_LOAD        => "LOAD",
        OP_STORE       => "STORE",
        OP_PUSH_R      => "PUSH_R",
        OP_POP_R       => "POP_R",
        OP_PRINT       => "PRINT",
        OP_PRINT_NOLF  => "PRINT_NL",
        OP_READLINE    => "READLINE",
        OP_STR_CONCAT  => "STR_CAT",
        OP_STR_LEN     => "STR_LEN",
        OP_STR_INTERP  => "STR_ITP",
        OP_TO_STRING   => "TO_STR",
        OP_ARR_NEW     => "ARR_NEW",
        OP_ARR_PUSH    => "ARR_PSH",
        OP_ARR_POP     => "ARR_POP",
        OP_ARR_GET     => "ARR_GET",
        OP_ARR_SET     => "ARR_SET",
        OP_ARR_LEN     => "ARR_LEN",
        OP_ARR_LITERAL => "ARR_LIT",
        OP_BUILTIN     => "BUILTIN",
        OP_SAVE        => "SAVE",
        OP_RESTORE     => "RESTORE",
        _ => "???",
    }
}
