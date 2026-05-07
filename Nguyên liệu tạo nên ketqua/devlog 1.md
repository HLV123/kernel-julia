# Nấc 1 — Ghép Tạng: ForthVM × MyKernel = ketqua

---

Có hai thứ nằm trên bàn. Một bên là MyKernel — 24 phases, đầy đủ từ bootloader đến TCP/IP stack, security hardening, chạy bare-metal trên x86_64, nhưng câm lặng hoàn toàn từ góc độ người dùng cuối. Bên kia là ForthVM — một máy ảo bytecode tự tay xây từng byte bằng Forth, có lexer, compiler, assembler, debugger, và một ngôn ngữ Julia Tiny đã chạy được 15 bài test liên tiếp. Nhưng ForthVM chỉ tồn tại trên host OS thông qua gforth.

Câu hỏi đặt ra không phải là *có nên ghép không* mà là *ghép như thế nào*.

---

## Quyết định đầu tiên: không viết lại từ đầu

Lựa chọn dễ nhất là thiết kế một VM mới hoàn toàn bằng Rust, lấy cảm hứng từ ForthVM. Nhưng cách đó sẽ mất đi thứ quan trọng nhất: mỗi dòng trong ForthVM đã được kiểm chứng. Từng opcode, từng nhánh của compiler, từng trường hợp edge case của lexer — tất cả đã chạy qua 15 bài test và một REPL tương tác.

Quyết định là **dịch**, không phải viết lại. Giữ nguyên logic, giữ nguyên cấu trúc, chỉ thay đổi ngôn ngữ và môi trường. Mỗi file `.fs` của ForthVM trở thành một file `.rs` tương ứng — mapping gần như 1-1.

---

## Những gì suôn sẻ

Phần lõi VM dịch sang rất tự nhiên. Forth và Rust đều có tư duy low-level rõ ràng — không có magic, mọi thứ đều explicit.

`00-constants.fs` với hàng loạt `constant` trở thành `pub const` trong `opcode.rs`. Bộ nhớ chia 4 vùng `[Code|Data|Stack|Heap]` trong mảng `program[1024]` — cấu trúc này dịch thẳng thành struct `VmMemory` với mảng `[u32; 1024]`. Stack pointer, call stack pointer, 8 thanh ghi — tất cả đều là field trong struct, không có gì bí ẩn.

Phần handlers dịch rất cơ học. Mỗi `: op-add ( -- ) vm-pop vm-pop + vm-push ;` trong Forth trở thành một function Rust vài dòng. Dispatch loop `begin running @ while fetch dispatch repeat` trở thành vòng `while self.running` quen thuộc.

Lexer và compiler cũng dịch được — logic recursive descent của `14-compiler.fs` giữ nguyên hoàn toàn sang `compiler.rs`. Cách Forth dùng `defer` cho đệ quy chéo giữa `jl-parse-expr` và `jl-parse-stmt` được giải quyết đơn giản bằng cách đặt cả hai trong cùng một `impl Compiler`.

---

## Những gì phải thích nghi

**Debugger bị bỏ.** `10-debugger.fs` là một trong những phần đẹp nhất của ForthVM gốc — vòng lặp `[n] bước tiếp / [r] chạy hết / [q] thoát` hiển thị bytecode, thanh ghi, stack side by side. Nhưng nó cần đọc bàn phím blocking, trong khi shell của MyKernel là async. Ghép vào sẽ block toàn bộ executor. Quyết định bỏ, để lại disassembler nhưng không có interactive stepping.

**String handling thay đổi hoàn toàn.** Forth làm việc với counted string `(addr, len)` trên stack theo cách rất tự nhiên với paradigm stack-based. Rust cần `&str`, `String`, `alloc::string::String`. Lexer và symbol table là chỗ tốn công nhất — toàn bộ `tok-id` buffer và `sym-names` array phải được bọc lại cho phù hợp với Rust string semantics.

**`frame_save.rs` là file không có trong ForthVM gốc.** Forth xử lý một số pattern đệ quy nhất định rất tự nhiên nhờ cấu trúc ngôn ngữ. Rust cần explicit frame stack để hỗ trợ mutual recursion đúng cách. Đây là file duy nhất được thêm mới mà không có file Forth tương ứng.

**I/O bridge.** ForthVM gốc dùng `.` và `cr` của gforth — print ra stdout host. Rust kernel có `println!` macro riêng đi qua VGA driver. `SyscallBridge` là lớp mỏng kết nối hai thế giới — VM gọi bridge, bridge gọi macro kernel.

---

## Kết nối với MyKernel: chỉ hai điểm chạm

Điều thú vị là dù là một phép ghép lớn về mặt khái niệm, điểm chạm thực sự giữa ForthVM và MyKernel rất ít:

**`src/lib.rs`** — một dòng: `pub mod forthvm;`

**`src/shell.rs`** — ba lệnh mới: `julia`, `vm`, `vmdemo`.

Toàn bộ 24 phases của MyKernel không bị chạm đến. ForthVM cắm vào như một organ mới mà không làm xáo trộn gì cả. Đây là lúc kiến trúc modular của MyKernel trả lời tốt nhất cho câu hỏi *tại sao phải xây kernel theo kiểu đó*.

---

## Lần đầu chạy `vmdemo`

Khi lệnh `vmdemo` cho ra output đúng lần đầu tiên — `5 + 6 = 11`, giai thừa `5! = 120`, Julia compiler tính `(2+3)*4 = 20` — cảm giác không hẳn là ngạc nhiên. Vì logic đã đúng từ ForthVM gốc. Nhưng lần này nó chạy trên bare-metal, không có OS, không có gforth, không có gì cả ngoài kernel tự viết. Đó là thứ khác.

---

## Giới hạn nhận ra ngay

ketqua chạy được, nhưng rõ ràng ngay từ đầu là nó chưa đủ để *dùng được*. Lệnh `julia` chỉ nhận một dòng lệnh từ shell — không thể viết hàm nhiều dòng vì shell split theo space. Muốn test giai thừa phải nhét hết vào một dòng với dấu `;` ngăn cách:

```
kernel> julia function fact(n) ; if n == 1 ; return 1 ; end ; return n * fact(n-1) ; end ; println(fact(6))
```

Không có string. Không có array. Không có boolean literal. Chỉ có số nguyên — đúng như ForthVM gốc, vì ForthVM gốc cũng chỉ có số nguyên. Và quan trọng nhất: không có REPL thực sự. ForthVM gốc *có* REPL (`[j] Julia REPL` trong menu), nhưng khi dịch sang Rust, REPL đó bị bỏ cùng với debugger vì cùng vấn đề blocking I/O.

ketqua là một thành công về mặt ghép tạng. Nhưng nó cũng là bản phác thảo rõ ràng về những gì cần làm tiếp theo.
