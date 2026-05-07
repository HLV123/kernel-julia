// ============================================================
// syscall_bridge.rs -- Lớp trung gian VM → Kernel
// Chuyển đổi VM operations sang kernel functions
//
// Trong kernel mode: gọi trực tiếp crate::print!, crate::fs::*
// Trong user mode (Ring 3): sẽ dùng syscall instruction
// ============================================================

use alloc::string::String;
use alloc::format;

/// Lớp trung gian kết nối VM với kernel
///
/// Khi VM chạy như kernel module, bridge gọi trực tiếp kernel APIs.
/// Khi VM chạy ở Ring 3, bridge sẽ phát syscall instructions.
pub struct SyscallBridge {
    /// Output buffer — lưu output cho testing/capture
    output_capture: bool,
    captured_output: String,
}

impl SyscallBridge {
    /// Tạo bridge mới (kernel mode — gọi trực tiếp)
    pub fn new() -> Self {
        SyscallBridge {
            output_capture: false,
            captured_output: String::new(),
        }
    }

    /// Tạo bridge với output capture (dùng cho testing)
    pub fn with_capture() -> Self {
        SyscallBridge {
            output_capture: true,
            captured_output: String::new(),
        }
    }

    /// Lấy captured output (dùng cho testing)
    pub fn get_captured_output(&self) -> &str {
        &self.captured_output
    }

    /// Xoá captured output
    pub fn clear_captured_output(&mut self) {
        self.captured_output.clear();
    }

    // --- OP_PRINT: In giá trị ra stdout ---
    // Forth: vm-pop . cr
    // Kernel: crate::print!("{}\n", value)

    /// In một giá trị i32 (signed) ra stdout
    pub fn print_value(&mut self, value: i32) {
        if self.output_capture {
            self.captured_output.push_str(&format!("{}\n", value));
        } else {
            crate::print!("{}\n", value);
        }
    }

    /// In chuỗi text ra stdout
    pub fn print_str(&mut self, s: &str) {
        if self.output_capture {
            self.captured_output.push_str(s);
        } else {
            crate::print!("{}", s);
        }
    }

    /// In chuỗi text với xuống dòng
    pub fn println_str(&mut self, s: &str) {
        if self.output_capture {
            self.captured_output.push_str(s);
            self.captured_output.push('\n');
        } else {
            crate::println!("{}", s);
        }
    }

    // --- OP_SAVE: Lưu trạng thái VM vào VFS ---
    // Forth gốc: chỉ in ra màn hình
    // Cải tiến: ghi vào file thật qua VFS

    /// Lưu dữ liệu binary vào file
    pub fn save_to_file(&self, path: &str, data: &[u8]) -> bool {
        match crate::fs::write_file(path, data) {
            Ok(()) => {
                crate::serial_println!("[forthvm] saved {} bytes to {}", data.len(), path);
                true
            }
            Err(e) => {
                crate::serial_println!("[forthvm] save error: {:?}", e);
                false
            }
        }
    }

    // --- OP_RESTORE: Đọc trạng thái VM từ VFS ---

    /// Đọc dữ liệu binary từ file
    pub fn load_from_file(&self, path: &str) -> Option<alloc::vec::Vec<u8>> {
        match crate::fs::read_file(path) {
            Ok(data) => {
                crate::serial_println!("[forthvm] loaded {} bytes from {}", data.len(), path);
                Some(data)
            }
            Err(e) => {
                crate::serial_println!("[forthvm] load error: {:?}", e);
                None
            }
        }
    }
}
