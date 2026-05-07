# Nấc 2 — Từ Proof-of-Concept Đến Runtime: ketqua → ketquahai

---

ketqua chứng minh được điều quan trọng nhất: ForthVM sống được trong kernel. Nhưng sống được và *dùng được* là hai chuyện khác nhau. Nấc hai bắt đầu từ câu hỏi thực tế: muốn ngồi gõ code Julia trên kernel này, cần thêm gì?

Câu trả lời không phải là một danh sách tính năng. Nó là một quyết định kiến trúc lớn đặt ra ngay từ đầu.

---

## Quyết định nền tảng: thay đổi kiểu dữ liệu

ForthVM gốc — và ketqua kế thừa — chỉ có một kiểu duy nhất: số nguyên `u32`. Mọi thứ là số. Stack là mảng `u32`. Thanh ghi là `u32`. Biến là `u32`. Cách này phù hợp hoàn toàn với Forth vì Forth bản thân cũng chỉ có cell — một word-size integer.

Nhưng Julia Tiny không phải Forth. Người dùng muốn viết `"hello $name"`, muốn có `[1, 2, 3]`, muốn `true` và `false` là những thứ thực sự, không phải `1` và `0`. Và quan trọng hơn — muốn `println` in ra chuỗi, không chỉ in số.

Giữ nguyên `u32` sẽ nghĩa là mọi tính năng mới đều là hack chồng lên hack. Quyết định là thay đổi ngay từ gốc: đưa vào một enum `Value` đa kiểu.

```rust
enum Value {
    Int(i32),
    Bool(bool),
    Str(StrId),
    Array(ArrId),
    Nil,
}
```

Đây là thay đổi phá vỡ nhất của toàn bộ nấc hai. Khi stack không còn là `[u32; 1024]` mà là `[Value; 1024]`, mọi handler đều phải viết lại. Mọi opcode arithmetic phải kiểm tra kiểu trước khi thực thi. VM giảm từ 248 dòng xuống 126 dòng vì bỏ được heap arena thủ công — bộ nhớ động giờ giao cho Rust `Vec` và hai pool quản lý.

`StringPool` và `ArrayPool` là hai cấu trúc mới: mỗi chuỗi và mỗi mảng được lưu một lần, tham chiếu bằng ID nguyên. Không bao giờ copy chuỗi khi push lên stack — chỉ copy ID. Đây là lý do tại sao `Value::Str(StrId)` thay vì `Value::Str(String)`.

---

## REPL: trả lại thứ đã mất ở nấc một

ForthVM gốc có REPL. ketqua mất nó vì vấn đề blocking I/O với async shell. ketquahai lấy lại bằng cách đi theo hướng khác — thay vì cố nhét REPL vào async executor, tạo một vòng lặp blocking riêng biệt chạy khi người dùng gõ `julia`.

Shell gọi `run_repl()`, hàm này chiếm quyền điều khiển hoàn toàn — đọc từng dòng qua serial, compile và chạy, lặp lại. Không async, không yield. Khi người dùng gõ `exit` thì trả quyền về cho shell.

Điểm quan trọng là REPL giữ nguyên `VarTable` và `FuncTable` giữa các lần nhập. Biến định nghĩa ở dòng trước còn đó ở dòng sau. Hàm viết lúc đầu session vẫn gọi được lúc cuối session. Đây là thứ `julia <code>` one-shot của ketqua không làm được — mỗi lần gọi là một VM mới, không có ký ức.

Multi-line cũng được xử lý: REPL đếm độ sâu của `if/while/for/function` — khi chưa đủ `end` thì in `...` và chờ dòng tiếp. Người dùng có thể gõ hàm nhiều dòng tự nhiên thay vì phải nhét tất cả vào một dòng với `;`.

`install_demo_files()` được gọi ngay khi vào REPL — tạo 11 file Julia script vào `/etc/julia/`. Đây là lần đầu tiên VFS của MyKernel được dùng thực sự từ góc độ người dùng, không chỉ từ kernel code.

---

## Compiler: khi một file không còn đủ

`compiler.rs` trong ketqua là 513 dòng — tất cả trong một file. Khi thêm `for`, `break`, `continue`, `+=`, string literal, array literal, built-in functions, `include`... con số đó sẽ vượt 1000 dòng và không thể maintain được.

Tách thành submodule `compiler/` với ba file:

- `mod.rs` — struct `Compiler`, emit, quản lý state, entry point `jl_run`
- `stmt.rs` — parse tất cả statements: if, while, for, function, break, continue, return, assignment
- `expr.rs` — parse tất cả expressions: số, string, bool, array, function call, operators

Cùng lúc đó, `Compiler` struct thay đổi quan trọng: VM không còn là `&'a mut ForthVm` borrow từ ngoài vào — nó là `ForthVm` owned bên trong compiler. Lý do thực tế: borrow checker không cho phép compiler vừa giữ mutable reference đến VM vừa gọi methods của chính mình theo một số pattern nhất định.

---

## Bug duy nhất chặn build

Trong toàn bộ quá trình phát triển ketquahai, chỉ có một lỗi compile thực sự — `E0499`, double mutable borrow trong `parse_continue`:

```rust
// Không compile được:
if let Some(ctx) = c.loop_stack.last_mut() {  // borrow #1
    let p = c.emit_jmp_placeholder()?;         // borrow #2 — lỗi
    ctx.continue_patches.push(p);
}
```

Rust không cho phép `c` bị mượn mutably hai lần cùng lúc — `ctx` đang giữ borrow #1 vào `c.loop_stack`, nhưng `emit_jmp_placeholder()` cần borrow toàn bộ `c`. Logic hoàn toàn đúng, chỉ là borrow checker không đủ thông minh để thấy hai borrow này không thực sự xung đột.

Fix bằng cách tách ra: đọc `continue_target` vào biến cục bộ trước, drop borrow, rồi mới gọi emit. Không thay đổi logic, chỉ thay đổi thứ tự.

Thú vị là đây cũng chính là lớp bài học mà Rust muốn dạy — code đúng logic chưa đủ, còn phải đúng về ownership. Và thường thì fix borrow checker lỗi cũng đồng thời làm code rõ ràng hơn.

---

## 35 built-in functions và câu hỏi về ranh giới

Khi thêm built-in functions, câu hỏi nảy sinh: thêm cái gì, bỏ cái gì?

ForthVM gốc không có built-in — mọi thứ là opcode. Nhưng trong một ngôn ngữ cấp cao hơn, `sqrt(144)`, `random()`, `write_file("/tmp/x", "hello")` không nên là opcodes. Chúng nên là function calls có tên.

Đáp án là `OP_BUILTIN` — một opcode duy nhất với argument là ID của hàm built-in. Compiler ánh xạ tên hàm sang ID lúc compile. Runtime dispatch sang implementation. 35 built-ins chia theo nhóm: toán học, chuỗi, mảng, hệ thống, file I/O.

File I/O là nhóm thú vị nhất — `read_file`, `write_file`, `file_exists` gọi thẳng vào VFS của MyKernel. Đây là điểm nối trực tiếp giữa ngôn ngữ script và kernel. Khi Julia code gọi `write_file("/tmp/note.txt", "hello")`, nó đang thực sự ghi vào RamFS đang chạy trong kernel — cùng filesystem mà `ls` và `cat` của shell đọc.

---

## Bỏ raw assembler và vmdemo

Một quyết định không phải thêm mà là bỏ: lệnh `vm` (raw assembly) và `vmdemo` (8 phase demo) bị xóa khỏi shell.

Lý do đơn giản: chúng thuộc về giai đoạn chứng minh khái niệm. `vm PUSH 5 PUSH 6 ADD PRINT HALT` là cách hay để demo VM hoạt động, nhưng không ai thực sự muốn viết chương trình bằng assembly tay. Và `vmdemo` chạy các demo bytecode thủ công — giờ có 11 file Julia script trong `/etc/julia/` làm tốt hơn nhiều với ngôn ngữ cấp cao.

Assembler vẫn còn trong codebase dưới dạng placeholder — không bị xóa hẳn, nhưng không được expose ra shell nữa.

---

## Test thực tế và những phát hiện

Khi file test 18 section chạy lần đầu, có ba vấn đề:

`fib(10)` trả về `-80`. Đệ quy Fibonacci với n=10 cần 177 lần gọi đệ quy — quá sâu cho call stack của VM. Giải pháp không phải tăng call stack mà là viết lại `fib` theo iterative. Đây cũng là bài học thực tế đầu tiên về giới hạn của runtime.

`is_prime` lặp vô tận, in `2` mãi không dừng. Nguyên nhân là biến `i` trong hàm `is_prime` dùng chung slot với biến `i` của vòng `for i = 1:20` bên ngoài — tất cả biến đều global, không có scope. Mỗi lần `is_prime` chạy xong, nó để lại `i` ở giá trị nào đó làm hỏng vòng lặp ngoài. Đổi tên biến trong hàm thành `d` giải quyết được, nhưng vấn đề thực sự sâu hơn — đây là limitation của thiết kế, không phải bug.

`ticks() > 0` trả về `false` vì `ticks()` = 0 lúc mới boot. Nhỏ nhưng đáng ghi lại: test assumptions cũng cần test.

Ba vấn đề này không phải bug nghiêm trọng. Nhưng chúng là bản đồ rõ ràng của những gì cần cải thiện ở nấc tiếp theo — scope biến, call stack depth, và behavior của system calls trong early boot.

---

## Trạng thái kết thúc nấc hai

Khi 18 section test pass, kernel prompt hiện:

```
=== ALL TESTS PASSED ===
jl>
```

Julia Tiny v0.2 chạy trên bare-metal x86_64, không có OS, không có gforth, không có gì ngoài kernel tự viết và VM tự dịch. Có REPL. Có string. Có array. Có boolean. Có file I/O thực sự vào VFS. Có 35 built-in functions. Có `for` loop với step và đếm ngược.

Và quan trọng nhất — có đủ để *dùng được*, không chỉ để demo.

Những gì còn thiếu vẫn rõ ràng: scope biến cục bộ, `~` bitwise NOT, số thực, dictionary. Đó là nấc ba — nếu có.
