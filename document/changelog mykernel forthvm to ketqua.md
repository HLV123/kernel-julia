# Changelog: mykernel + ForthVM → ketqua

> Ghép tạng: cấy ForthVM (viết bằng Forth/gforth) vào MyKernel (viết bằng Rust/no_std),  
> biến một OS kernel thuần túy thành một kernel có khả năng lập trình được.

---

## Bối cảnh: Hai dự án độc lập

### MyKernel
Một OS kernel hoàn chỉnh 24 phases viết bằng Rust bare-metal — có đủ memory management, VFS, TCP/IP, scheduler, security. Nhưng chỉ là hạ tầng — không có ngôn ngữ script, không thể lập trình từ bên trong.

### ForthVM
Một máy ảo bytecode tự viết bằng **gforth** (Forth dialect), chạy trên host OS, với mục tiêu kiểm soát từng byte để có ngôn ngữ Julia Tiny. Gồm 18 file `.fs`:

```
00-constants.fs   — Hằng số, opcode, phân vùng bộ nhớ
01-memory.fs      — 4 vùng nhớ chính + biến trạng thái
02-stack.fs       — Data stack
03-registers.fs   — 8 thanh ghi R0–R7
04-program.fs     — Program memory + fetch
05-callstack.fs   — Call stack
06-segments.fs    — Data segment + Heap arena
07-handlers.fs    — Xử lý từng opcode
08-dispatch.fs    — Vòng lặp fetch-dispatch
09-disasm.fs      — Disassembler
10-debugger.fs    — Debugger tương tác (step/run/quit)
11-assembler.fs   — Text assembler ("PUSH 5 ADD HALT" → bytecode)
12-lexer.fs       — Lexer Julia Tiny
13-symbols.fs     — Bảng biến + bảng hàm
14-compiler.fs    — Recursive descent compiler Julia → bytecode
15-demos.fs       — Menu demo 8 phase
run.fs            — Chạy 15 bài Julia tự động
main.fs           — Entry point, include tất cả, gọi run-menu
```

Chạy bằng: `gforth main.fs`

---

## Quyết định ghép tạng

ForthVM đã có một ngôn ngữ Julia Tiny hoạt động hoàn chỉnh — lexer, compiler, VM, assembler, debugger. Nhưng nó chỉ chạy được trên host OS thông qua gforth.

MyKernel đã có đủ hạ tầng OS — VFS, memory, I/O, shell — nhưng thiếu khả năng lập trình.

Giải pháp: **dịch toàn bộ ForthVM từ Forth sang Rust no_std**, cắm vào kernel như một module, kết nối với VFS và shell sẵn có.

---

## Quá trình ghép: Forth → Rust no_std

### Nguyên tắc dịch

Mỗi file `.fs` của ForthVM trở thành một file `.rs` tương ứng trong `src/forthvm/`:

| ForthVM (Forth) | ketqua (Rust) | Ghi chú |
|----------------|--------------|---------|
| `00-constants.fs` | `opcode.rs` | `constant` → `pub const` |
| `01-memory.fs` | `memory.rs` | `create` array → Rust struct + arrays |
| `02-stack.fs` | `stack.rs` | `variable vsp` → `sp: usize` field |
| `03-registers.fs` | `registers.rs` | `create reg` → `[u32; 8]` |
| `04-program.fs` | `vm.rs` (một phần) | `vpc`, `fetch` → `pc`, `fetch()` method |
| `05-callstack.fs` | `callstack.rs` | `variable vcsp` → `csp: usize` field |
| `06-segments.fs` | `memory.rs` (mở rộng) | `data@/data!` → `data_read/write()` |
| `07-handlers.fs` | `handlers.rs` | Mỗi `op-xxx` → `fn op_xxx()` |
| `08-dispatch.fs` | `vm.rs` + `handlers.rs` | `dispatch` → `match opcode` |
| `09-disasm.fs` | `disasm.rs` | `disasm-one` → `disasm_one()` |
| `10-debugger.fs` | *(bỏ)* | Debugger tương tác không phù hợp no_std |
| `11-assembler.fs` | `assembler.rs` | `asm-eval` → `assemble_into()` |
| `12-lexer.fs` | `lexer.rs` | `jl-next` → `Lexer::next()` |
| `13-symbols.fs` | `symbols.rs` | `sym-find-or-add` → `VarTable` |
| `14-compiler.fs` | `compiler.rs` | `jl-run` → `jl_run()` |
| `15-demos.fs` | `demos.rs` | `demo-1..8` → Rust functions |
| `run.fs` | *(tích hợp vào demos)* | 15 bài test → `run_all_demos()` |
| `main.fs` | `mod.rs` | `include` chain → `pub mod` |

### Thách thức khi dịch

**1. Không có gforth runtime**  
Forth dùng `create`, `allot`, `variable`, `constant` để quản lý bộ nhớ tự do trên host. Rust no_std không có allocator mặc định — mọi thứ phải dùng `alloc::` crate với heap kernel đã thiết lập từ Phase 7.

**2. Không có I/O trực tiếp**  
Forth dùng `.` và `cr` in ra stdout. Rust kernel dùng macro `println!` / `print!` tự viết qua VGA driver.

**3. String handling**  
Forth dùng counted strings `(addr, len)` trên stack. Rust dùng `&str`, `String`, `alloc::string::String` — cần bọc lại toàn bộ lexer và symbol table.

**4. `defer` / `:noname` / đệ quy chéo**  
Compiler Forth dùng `defer` để khai báo trước 2 hàm đệ quy chéo (`jl-parse-expr` ↔ `jl-parse-stmt`). Rust giải quyết bằng cách đặt cả hai trong cùng một `impl Compiler` block.

**5. Debugger bỏ qua**  
`10-debugger.fs` có vòng lặp đọc bàn phím tương tác (`read-cmd`) không tương thích với kiến trúc async shell của kernel. Được bỏ qua trong phiên bản này.

**6. Frame stack mới thêm**  
`frame_save.rs` không có trong ForthVM gốc — được thêm vào để hỗ trợ đệ quy đôi (mutual recursion) trong Rust, vì Rust không có `defer` như Forth.

**7. `state_persist.rs`**  
`op-restore` trong Forth chỉ in ra "chưa hỗ trợ". Giữ placeholder trong Rust, cấu trúc để phát triển sau.

---

## Kết quả: Module `forthvm` trong ketqua

### Cấu trúc module

```
src/forthvm/
├── mod.rs          — pub mod + re-export
├── opcode.rs       — 27 opcodes + hằng số bộ nhớ
├── memory.rs       — VmMemory: program[1024] + data + heap
├── stack.rs        — DataStack: [u32; 1024]
├── callstack.rs    — CallStack: [usize; 256]
├── registers.rs    — Registers: [u32; 8]
├── frame_save.rs   — FrameStack (mới, không có trong ForthVM gốc)
├── syscall_bridge.rs — VM → kernel I/O bridge
├── vm.rs           — ForthVm struct + fetch/dispatch loop
├── handlers.rs     — 27 opcode handlers
├── disasm.rs       — Disassembler
├── assembler.rs    — Text assembler
├── lexer.rs        — Lexer Julia Tiny
├── symbols.rs      — VarTable + FuncTable
├── compiler.rs     — Recursive descent compiler
├── state_persist.rs — Placeholder
└── demos.rs        — 6 demo functions
```

### Kiến trúc VM sau khi ghép

```
Nguồn Julia/Assembly (chuỗi &str)
          ↓
     Lexer (từ 12-lexer.fs)
          ↓  tokens
     Compiler (từ 14-compiler.fs)
     Assembler (từ 11-assembler.fs)
          ↓  bytecode u32
     VmMemory.program[1024]
          ↓
     ForthVm.run() (từ 08-dispatch.fs)
          ↓  fetch → dispatch
     Handlers (từ 07-handlers.fs)
          ↓
     SyscallBridge → kernel println!
```

### Kiểu dữ liệu

Toàn bộ VM chỉ có **một kiểu duy nhất: `u32`** — giống ForthVM gốc. Stack, registers, memory đều là mảng `u32`. Chưa có string, bool, array, nil — đây là giới hạn được kế thừa nguyên vẹn từ ForthVM Forth.

### Instruction set (27 opcodes — giống hệt ForthVM gốc)

| Nhóm | Opcodes |
|------|---------|
| Stack | `PUSH`, `DUP`, `DROP`, `SWAP` |
| Thanh ghi | `PUSH_R`, `POP_R` |
| Số học | `ADD`, `SUB`, `MUL` |
| So sánh | `CMP_EQ`, `CMP_GT` |
| Nhảy | `JMP`, `JZ`, `JGT` |
| Hàm | `CALL`, `RET`, `FRAME_SAVE` |
| Bộ nhớ | `LOAD_DATA`, `STORE_DATA`, `ALLOC`, `FREE`, `HEAP_LOAD`, `HEAP_STORE` |
| I/O | `PRINT` |
| Điều khiển | `HALT`, `SAVE`, `RESTORE` |

### Tính năng ngôn ngữ Julia Tiny (kế thừa từ 14-compiler.fs)

| Tính năng | Từ ForthVM gốc |
|----------|---------------|
| Số nguyên | ✅ |
| Biến toàn cục | ✅ |
| `+ - *` | ✅ (chưa có `/`, `%`, `^`) |
| `== != > < >= <=` | ✅ |
| `if / elseif / else / end` | ✅ |
| `while / end` | ✅ |
| `function / return` | ✅ |
| Gọi hàm với tham số | ✅ |
| Đệ quy | ✅ |
| `println(expr)` | ✅ |
| Dấu `;` thay cho newline | ✅ |

Chưa có (cũng chưa có trong ForthVM gốc): `for`, `break`, `continue`, `print`, string, bool, array, `+=`, built-in functions, REPL.

---

## Kết nối với MyKernel

### `src/lib.rs`

Thêm 1 dòng khai báo module:

```rust
/// Julia-Forth VM — Phase 25: Userland Runtime.
pub mod forthvm;
```

### `src/shell.rs`

Thêm 3 lệnh mới vào shell, tận dụng VFS và I/O sẵn có của kernel:

**`julia <code>`** — chạy Julia one-shot:
```
kernel> julia println((2 + 3) * 4)
20
kernel> julia function fact(n) ; if n == 1 ; return 1 ; end ; return n * fact(n-1) ; end ; println(fact(6))
720
```

**`vm <asm>`** — chạy raw assembly bytecode (tương đương `asm-eval` + `vm-run` của ForthVM):
```
kernel> vm PUSH 5 PUSH 6 ADD PRINT HALT
11
kernel> vm PUSH 10 PUSH 3 MUL PRINT HALT
30
```

**`vmdemo`** — chạy 6 demo tương ứng với 8 phase của ForthVM gốc:
```
kernel> vmdemo
=== Phase 1: Tinh 5 + 6 ===
=== Phase 2: Dem nguoc 5 -> 1 ===
=== Phase 3: Giai thua 5! = 120 ===
=== Phase 5: Bo nho Data va Heap ===
=== Phase 6: Hop dich van ban ===
=== Phase 8: Ngon ngu Julia Tiny ===
```

### Điểm kết nối quan trọng: VFS

Khi ForthVM gốc chỉ có `op-print` in ra stdout của gforth, phiên bản Rust dùng `SyscallBridge` để gọi macro `println!` của kernel — output ra VGA và serial, cùng pipeline với toàn bộ kernel output.

---

## Những gì giữ nguyên từ MyKernel

Tất cả 41 file `.rs` của mykernel không bị chạm đến. Toàn bộ 24 phases — VGA, interrupts, heap, scheduler, Ring 3, VFS, initramfs, FAT32, syscalls, APIC, SMP, TCP/IP, socket, security — hoạt động nguyên vẹn.

---

## So sánh: ForthVM gốc vs ForthVM trong ketqua

| Khía cạnh | ForthVM gốc (Forth) | ketqua (Rust no_std) |
|-----------|--------------------|--------------------|
| Chạy trên | gforth / host OS | MyKernel bare-metal |
| Ngôn ngữ viết | Forth (gforth) | Rust no_std |
| I/O | stdout gforth | VGA + serial kernel |
| Debugger | ✅ tương tác | ❌ bỏ (không tương thích async) |
| Assembler | ✅ đầy đủ | ✅ giữ nguyên |
| Compiler Julia | ✅ | ✅ giữ nguyên |
| Kiểu dữ liệu | u32 (Forth cell) | u32 |
| Tích hợp VFS | ❌ | ✅ (qua SyscallBridge) |
| Gọi từ shell | ❌ | ✅ (`julia`, `vm`, `vmdemo`) |
| String/Array/Bool | ❌ | ❌ (chưa, phase sau) |

---

*Phân tích dựa trên source code ForthVM.zip và diff giữa mykernel.zip và ketqua.zip*
