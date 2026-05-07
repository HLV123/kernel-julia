# Changelog: ketqua → ketquahai

> Tái kiến trúc toàn diện từ VM số nguyên đơn giản sang runtime đa kiểu đầy đủ với REPL tương tác

---

## Tổng quan

| | ketqua | ketquahai |
|--|--------|-----------|
| Số file `.rs` trong `forthvm/` | 17 | 22 (+5) |
| Kiểu dữ liệu | `u32` (chỉ số nguyên) | `Int`, `Bool`, `String`, `Array`, `Nil` |
| REPL tương tác | ❌ | ✅ |
| Load file `.jl` | ❌ | ✅ |
| String | ❌ | ✅ |
| Array | ❌ | ✅ |
| Bool (`true`/`false`) | ❌ | ✅ |
| `for` loop + range | ❌ | ✅ |
| `break` / `continue` | ❌ | ✅ |
| `print` (không xuống dòng) | ❌ | ✅ |
| `+=` `-=` `*=` `/=` `%=` | ❌ | ✅ |
| Built-in functions | ❌ | ✅ 35 hàm |
| Lệnh `vm` (raw assembly) | ✅ | ❌ (bỏ) |
| Lệnh `vmdemo` (8 phase demo) | ✅ | ❌ (bỏ) |
| Demo files `/etc/julia/` | Bytecode demo | 11 file Julia script |
| Program size | 256 instructions | 4096 instructions (×16) |

---

## 1. File mới hoàn toàn

### `src/forthvm/value.rs` — Hệ thống kiểu động

Thay thế kiểu `u32` đơn giản bằng enum `Value` đa kiểu:

```rust
pub enum Value {
    Int(i32),       // Số nguyên có dấu
    Bool(bool),     // true / false
    Str(StrId),     // Tham chiếu vào StringPool
    Array(ArrId),   // Tham chiếu vào ArrayPool
    Nil,            // Giá trị rỗng
}
```

Hai pool quản lý bộ nhớ:
- **`StringPool`**: `Vec<String>` — mỗi chuỗi được lưu 1 lần, tham chiếu bằng `StrId (u32)`
- **`ArrayPool`**: `Vec<Vec<Value>>` — mỗi mảng là `Vec<Value>`, tham chiếu bằng `ArrId (u32)`

Hàm tiện ích:
- `as_int()` — chuyển sang `i32` (Bool → 0/1)
- `is_truthy()` — `0`, `false`, `Nil` = false; còn lại = true
- `type_name()` — trả về `"Int"`, `"Bool"`, `"String"`, `"Array"`, `"Nil"`
- `format_value()` — chuyển `Value` → chuỗi in được

---

### `src/forthvm/builtins.rs` — 35 built-in functions

Toàn bộ built-in được đánh ID và dispatch qua opcode `OP_BUILTIN`:

**Toán học (7 hàm):**
`abs`, `max`, `min`, `sqrt`, `gcd`, `sign`, `clamp`

**Chuỗi (10 hàm):**
`length`, `uppercase`, `lowercase`, `string`, `parse_int`, `startswith`, `endswith`, `contains`, `repeat`, `char`, `ascii`

**Mảng (5 hàm):**
`sum`, `maximum`, `minimum`, `sort!`, `reverse!`

**Hệ thống (5 hàm):**
`ticks`, `random`, `uptime`, `sleep`, `heap_free`

**File I/O (4 hàm):**
`read_file`, `write_file`, `append_file`, `file_exists`

**Mảng (2 hàm):**
`push!`, `pop!`

Mỗi builtin có:
- Hằng số ID (`BI_ABS = 0`, `BI_MAX = 1`, ...)
- Số tham số khai báo trong `builtin_param_count()`
- Logic thực thi trong `exec_builtin()`

---

### `src/forthvm/repl.rs` — REPL tương tác

Module mới hoàn toàn. Là tính năng quan trọng nhất của ketquahai.

**Chức năng:**
- Vòng lặp `jl>` đọc từng dòng input từ serial/keyboard
- Gọi `install_demo_files()` lúc khởi động REPL (tạo `/etc/julia/`)
- Xử lý multi-line: đếm `depth` của `if/while/for/function/end` để ghép nhiều dòng thành 1 khối
- Giữ nguyên `VarTable` và `FuncTable` giữa các lần nhập — biến và hàm persist suốt session

**Lệnh REPL đặc biệt:**

| Lệnh | Tác dụng |
|------|---------|
| `exit` | Thoát REPL |
| `help` | Hiện danh sách tính năng |
| `vars` | Liệt kê tất cả biến hiện tại |
| `funcs` | Liệt kê tất cả hàm đã định nghĩa |
| `clear` | Reset toàn bộ VM state |
| `include("path")` | Load và chạy file `.jl` |

**`run_file_oneshot(path)`**: chạy file `.jl` không cần REPL, dùng cho `julia /path/to/file.jl`.

---

### `src/forthvm/compiler/` — Tái cấu trúc compiler thành submodule

`compiler.rs` (file đơn) → `compiler/` (thư mục 3 file):

| File | Nội dung |
|------|---------|
| `mod.rs` | `Compiler` struct, `emit`, `jl_run`, quản lý state |
| `stmt.rs` | Parse statements: `if`, `while`, `for`, `function`, `break`, `continue`, `return`, assignment |
| `expr.rs` | Parse expressions: số, string, bool, array literal, call, operators |

**`Compiler` struct mới** (so với ketqua):

```rust
pub struct Compiler {
    lexer: Lexer,
    vm: ForthVm,           // Owned (không borrow nữa)
    vars: VarTable,
    funcs: FuncTable,
    emit_ptr: usize,
    loop_stack: Vec<LoopCtx>,   // MỚI: theo dõi break/continue patches
    is_expr_stmt: bool,          // MỚI: auto-print trong REPL
}
```

`LoopCtx` lưu địa chỉ cần patch cho `break` và `continue`:
```rust
struct LoopCtx {
    break_patches: Vec<usize>,
    continue_patches: Vec<usize>,
    continue_target: Option<usize>,
}
```

---

## 2. File được sửa đổi lớn

### `src/forthvm/lexer.rs` (+3916 bytes — gần gấp đôi)

Từ 18 token → **40+ token**. Bổ sung:

**Toán tử mới:**
- `+=`, `-=`, `*=`, `/=`, `%=` — compound assignment
- `/`, `%`, `^` — chia, chia dư, lũy thừa
- `&&`, `||`, `!` — logic
- `&`, `|`, `~`, `<<`, `>>` — bitwise
- `[`, `]` — array indexing

**Từ khoá mới:**
`for`, `break`, `continue`, `print`, `true`, `false`, `local`, `include`

**Kiểu token mới:**
- `StringLit` — chuỗi trong dấu `"..."` với xử lý escape `\n`, `\t`, `\\`
- `HexNum` — số hex `0xFF`
- `Dollar` — cho string interpolation `$var`
- Thêm `col` và `line` vào mỗi token (thông tin vị trí để báo lỗi)

---

### `src/forthvm/opcode.rs` (+928 bytes)

Mở rộng instruction set từ 27 → **55 opcodes**. Tăng `PROG_SIZE` từ 1024 → **4096** (×4).

**Opcodes mới theo nhóm:**

| Nhóm | Opcodes mới |
|------|------------|
| Stack | `PUSH_INT`, `PUSH_TRUE`, `PUSH_FALSE`, `PUSH_NIL`, `PUSH_STR` |
| Số học | `DIV`, `MOD`, `POW`, `NEG` |
| So sánh | `CMP_NEQ`, `CMP_LT`, `CMP_LTE`, `CMP_GTE`, `JNZ` |
| Logic | `AND`, `OR`, `NOT` |
| Bitwise | `BAND`, `BOR`, `BXOR`, `SHL`, `SHR` |
| I/O | `PRINT_NOLF` (print không xuống dòng), `READLINE` |
| Chuỗi | `STR_CONCAT`, `STR_LEN`, `STR_INTERP`, `TO_STRING` |
| Mảng | `ARR_NEW`, `ARR_PUSH`, `ARR_POP`, `ARR_GET`, `ARR_SET`, `ARR_LEN`, `ARR_LITERAL` |
| Built-in | `BUILTIN` (arg = builtin ID) |

Thêm `unpack_arg_signed()` và `pack_signed()` cho jump offset có dấu.

---

### `src/forthvm/handlers.rs` (+2970 bytes)

Viết lại toàn bộ để làm việc với `Value` thay vì `u32`:

- `VmState` bổ sung `strings: &mut StringPool` và `arrays: &mut ArrayPool`
- Mọi `push`/`pop` đều xử lý `Value` enum
- Các handler số học kiểm tra kiểu trước khi thực thi (Int × Int, Str × Int cho repeat...)
- Handler `OP_BUILTIN`: pop N tham số, gọi `exec_builtin()`, push kết quả
- Handler mảng: `ARR_NEW`, `ARR_GET`, `ARR_SET`, `ARR_PUSH`, `ARR_POP`, `ARR_LEN`, `ARR_LITERAL`
- Handler chuỗi: concat, interp, to_string
- Xử lý `JZ`/`JNZ` dựa trên `is_truthy()` thay vì so sánh với 0

---

### `src/forthvm/vm.rs` (−5089 bytes — đơn giản hóa)

Loại bỏ `prog_end` và các phương thức phức tạp. Thêm:
- `strings: StringPool` và `arrays: ArrayPool` vào struct
- `reset_code()` — reset code/stack nhưng giữ string/array pools (cho REPL)
- `reset_all()` — reset hoàn toàn
- `run_and_get_top()` — chạy và trả về đỉnh stack (cho REPL auto-print)
- `instr_count: u64` — đếm số lệnh đã thực thi

---

### `src/forthvm/symbols.rs`

Tách `MAX_PARAMS` thành hằng số public. `VarTable` và `FuncTable` giờ làm việc với `Value` slots thay vì `u32` slots.

---

### `src/forthvm/demos.rs` (viết lại)

Từ demo bytecode thủ công → **11 file Julia script** cài vào `/etc/julia/`:

| File | Nội dung |
|------|---------|
| `hello.jl` | Hello World |
| `arithmetic.jl` | Các phép toán + `abs()` |
| `fibonacci.jl` | Fibonacci đệ quy |
| `factorial.jl` | Giai thừa 1–10 |
| `fizzbuzz.jl` | FizzBuzz 1–30 |
| `strings.jl` | String, nội suy, `repeat()` |
| `arrays.jl` | Mảng, `push!`, `sum`, `sort!` |
| `forloop.jl` | For range, step, đếm ngược |
| `fileio.jl` | Đọc/ghi file `/tmp/` |
| `system.jl` | `uptime`, `ticks`, `random` |
| `primes.jl` | Số nguyên tố đến 50 |

`run_all_demos()` bị xóa — không còn demo bytecode thủ công.  
`install_demo_files()` được gọi từ `run_repl()` mỗi khi vào REPL.

---

### Các file đơn giản hóa (shrink)

Các file sau bị giảm kích thước do loại bỏ tính năng heap arena, phân vùng bộ nhớ phức tạp, và raw assembler:

| File | ketqua | ketquahai | Lý do |
|------|--------|-----------|-------|
| `assembler.rs` | 6949 bytes | 648 bytes | Chỉ còn placeholder, không dùng |
| `memory.rs` | 3873 bytes | 1833 bytes | Bỏ heap arena, bộ nhớ đơn giản hơn |
| `state_persist.rs` | 3644 bytes | 269 bytes | Bỏ logic persist phức tạp |
| `callstack.rs` | 1759 bytes | 828 bytes | Đơn giản hóa |
| `frame_save.rs` | 2888 bytes | 1293 bytes | Đơn giản hóa |
| `syscall_bridge.rs` | 3521 bytes | 827 bytes | Đơn giản hóa |

---

### `src/shell.rs` (−1539 bytes)

**Xóa 2 lệnh:**
- `vm <asm>` — raw VM assembly (không còn dùng assembler)
- `vmdemo` — 8 phase bytecode demo

**Sửa lệnh `julia`** — từ one-shot → đa chế độ:

| Cú pháp | ketqua | ketquahai |
|---------|--------|-----------|
| `julia` (không arg) | In usage | Vào **REPL tương tác** `jl>` |
| `julia /path/file.jl` | Không hỗ trợ | Chạy file `.jl` |
| `julia <code>` | Chạy one-shot | Chạy one-shot |

Cập nhật `cmd_help()`:
```
julia             -- enter Julia REPL (interactive)
julia <file.jl>   -- run a Julia script file
julia <code>      -- run Julia code directly

Julia demo files:  ls /etc/julia
```

---

## 3. Tính năng ngôn ngữ bổ sung (so sánh compiler)

| Tính năng | ketqua | ketquahai |
|----------|--------|-----------|
| Số nguyên | ✅ `u32` | ✅ `i32` (có dấu) |
| Số âm literal | ❌ | ✅ `-42` |
| Boolean | ❌ | ✅ `true`, `false` |
| String | ❌ | ✅ `"hello"` |
| String interpolation | ❌ | ✅ `"$name"` |
| Array literal | ❌ | ✅ `[1, 2, 3]` |
| Array index | ❌ | ✅ `a[i]` |
| Nil | ❌ | ✅ |
| `/`, `%`, `^` | ❌ | ✅ |
| `&&`, `\|\|`, `!` | ❌ | ✅ |
| `&`, `\|`, `<<`, `>>` | ❌ | ✅ |
| `+=`, `-=`, `*=`, `/=`, `%=` | ❌ | ✅ |
| `for i=1:n` | ❌ | ✅ |
| `for i=a:step:b` | ❌ | ✅ |
| `break` | ❌ | ✅ |
| `continue` | ❌ | ✅ |
| `print` (no newline) | ❌ | ✅ |
| `println` nhiều args | ❌ | ✅ |
| Built-in functions | ❌ | ✅ 35 hàm |
| File I/O | ❌ | ✅ |
| REPL tương tác | ❌ | ✅ |
| Load file `.jl` | ❌ | ✅ |
| Hex number `0xFF` | ❌ | ✅ |

---

## 4. Bug được fix

### E0499 — Double mutable borrow trong `parse_continue`

**File:** `src/forthvm/compiler/stmt.rs`

**Nguyên nhân:** `ctx` đang giữ `&mut c.loop_stack`, nhưng `c.emit_jmp_placeholder()` cũng cần `&mut c` → conflict.

**Fix:** Tách thành 3 bước tuần tự — kiểm tra empty → đọc `continue_target` vào biến cục bộ → emit hoặc patch, tránh hai borrow cùng lúc.

---

## 5. Kiến trúc thay đổi cốt lõi

### Bộ nhớ

| Aspect | ketqua | ketquahai |
|--------|--------|-----------|
| Stack element | `u32` | `Value` enum |
| String storage | Không có | `StringPool` (intern) |
| Array storage | Không có | `ArrayPool` (heap) |
| Heap arena | Manual (`ALLOC`/`FREE`) | Tự động qua Rust `Vec` |
| Program size | 1024 cells, code chỉ 256 | 4096 cells |

### Compiler

| Aspect | ketqua | ketquahai |
|--------|--------|-----------|
| Cấu trúc | 1 file `compiler.rs` | Submodule `compiler/` (3 file) |
| VM ownership | `&'a mut ForthVm` (borrow) | `ForthVm` (owned) |
| Loop tracking | Không có | `loop_stack: Vec<LoopCtx>` |
| Compile error | Cơ bản | Có thêm vị trí line/col |

---

*Phân tích dựa trên diff source code giữa ketqua.zip và ketquahai.zip*
