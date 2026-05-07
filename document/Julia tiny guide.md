# Julia Tiny v0.2 — Hướng dẫn viết code

> **môi trường**
> rustc 1.97.0-nightly (365c0e1d7 2026-05-06)
> cargo 1.97.0-nightly (4f9b52075 2026-05-01)
> bootimage 0.10.4
> QEMU emulator version 11.0.50 (v11.0.0-12631-g54e84cdc7a)

> **vào folder ketquahai mở powershell và chạy lần lượt**
> cargo build
> cargo bootimage
> qemu-system-x86_64 -drive format=raw,file=target\x86_64-mykernel\debug\bootimage-mykernel.bin -serial stdio -no-reboot

> Chạy trên **MyKernel** · REPL: `kernel> julia` 
> Load file: `include("/tmp/file.jl")`

---

## Mục lục

1. [Khởi động REPL](#1-khởi-động-repl)
2. [Kiểu dữ liệu](#2-kiểu-dữ-liệu)
3. [Biến & phép gán](#3-biến--phép-gán)
4. [Toán tử](#4-toán-tử)
5. [Chuỗi](#5-chuỗi)
6. [Mảng](#6-mảng)
7. [Câu lệnh điều kiện](#7-câu-lệnh-điều-kiện)
8. [Vòng lặp while](#8-vòng-lặp-while)
9. [Vòng lặp for](#9-vòng-lặp-for)
10. [Hàm](#10-hàm)
11. [Hàm toán học](#11-hàm-toán-học)
12. [Hàm hệ thống](#12-hàm-hệ-thống)
13. [File I/O](#13-file-io)
14. [In ra màn hình](#14-in-ra-màn-hình)
15. [Demo files có sẵn](#15-demo-files-có-sẵn)
16. [Bytecode VM — kiến trúc nội bộ](#16-bytecode-vm--kiến-trúc-nội-bộ)
17. [Lệnh REPL](#17-lệnh-repl)
18. [Giới hạn & lưu ý](#18-giới-hạn--lưu-ý)
19. [Tính năng bổ sung](#19-tính-năng-bổ-sung)
20. [Ví dụ đầy đủ](#20-ví-dụ-đầy-đủ)

---

## 1. Khởi động REPL

```
kernel> julia
```

Thoát REPL:
```
jl> exit
```

Load và chạy file `.jl`:
```
jl> include("/tmp/myfile.jl")
```

> ⚠️ **Bắt buộc dùng đường dẫn tuyệt đối** bắt đầu bằng `/`.  
> File phải được nhúng vào initramfs trước khi build (xem phần 16).

---

## 2. Kiểu dữ liệu

| Kiểu | Ví dụ | Ghi chú |
|------|-------|---------|
| Số nguyên | `42`, `-7`, `0` | 64-bit signed |
| Boolean | `true`, `false` | |
| Chuỗi | `"hello"` | Dùng dấu nháy kép |
| Mảng | `[1, 2, 3]` | Index bắt đầu từ **1** |
| Nil | `nil` | Giá trị rỗng / lỗi |

---

## 3. Biến & phép gán

### Gán thường
```julia
x = 10
name = "Alice"
flag = true
arr = [1, 2, 3]
```

### Gán kết hợp
```julia
x += 5    # x = x + 5
x -= 3    # x = x - 3
x *= 2    # x = x * 2
x /= 4    # x = x / 4
x %= 3    # x = x % 3
```

> ⚠️ **Không có `local`** — tất cả biến đều là global.  
> Biến trong hàm **có thể xung đột** với biến bên ngoài cùng tên (xem phần 16).

---

## 4. Toán tử

### Số học
```julia
10 + 3   # 13
10 - 3   # 7
10 * 3   # 30
10 / 3   # 3  (chia nguyên)
10 % 3   # 1  (chia dư)
2 ^ 10   # 1024 (lũy thừa)
```

### So sánh
```julia
5 == 5   # true
5 != 4   # true
5 > 3    # true
5 < 3    # false
5 >= 5   # true
5 <= 4   # false
```

### Logic
```julia
true && false   # false
true || false   # true
!true           # false
```

### Bitwise
```julia
6 & 3    # 2   (AND)
6 | 3    # 7   (OR)
1 << 3   # 8   (shift trái)
16 >> 2  # 4   (shift phải)
```

> ⚠️ Toán tử `~x` (bitwise NOT unary) **chưa được hỗ trợ**.

---

## 5. Chuỗi

### Khai báo
```julia
s = "hello world"
```

### Nội suy biến (string interpolation)
```julia
name = "Julia"
println("Hello $name")      # Hello Julia
println("1 + 1 = $(1+1)")   # KHÔNG hỗ trợ biểu thức trong $()
```

> ⚠️ Chỉ hỗ trợ `$tên_biến` đơn giản, **không** hỗ trợ `$(biểu thức)`.

### Hàm chuỗi
```julia
length("hello")          # 5
uppercase("hello")       # HELLO
lowercase("HELLO")       # hello
```

---

## 6. Mảng

### Khai báo & truy cập
```julia
a = [10, 20, 30]
println(a[1])    # 10  (index bắt đầu từ 1)
println(a[3])    # 30
```

### Gán phần tử
```julia
a[2] = 99
println(a[2])    # 99
```

### Hàm mảng
```julia
push!(a, 40)        # thêm vào cuối
length(a)           # số phần tử
sum(a)              # tổng các phần tử
maximum(a)          # giá trị lớn nhất
minimum(a)          # giá trị nhỏ nhất
sort!(a)            # sắp xếp tăng dần (in-place)
reverse!(a)         # đảo ngược (in-place)
pop!(a)             # xóa phần tử cuối
```

### Duyệt mảng
```julia
a = [10, 20, 30]
for i = 1:length(a)
    println(a[i])
end
```

---

## 7. Câu lệnh điều kiện

### if / end
```julia
if x > 0
    println("dương")
end
```

### if / else / end
```julia
if x > 0
    println("dương")
else
    println("không dương")
end
```

### if / elseif / else / end
```julia
if x > 0
    println("dương")
elseif x < 0
    println("âm")
else
    println("bằng 0")
end
```

> Có thể lồng nhiều `elseif` tùy ý.  
> **Bắt buộc có `end`** để đóng khối.

---

## 8. Vòng lặp while

```julia
i = 1
while i <= 5
    println(i)
    i += 1
end
```

### break — thoát vòng lặp
```julia
i = 0
while true
    i += 1
    if i == 3
        break
    end
    println(i)
end
# In: 1  2
```

### continue — bỏ qua iteration hiện tại
```julia
i = 0
while i < 5
    i += 1
    if i == 3
        continue
    end
    println(i)
end
# In: 1  2  4  5
```

---

## 9. Vòng lặp for

### Duyệt theo range
```julia
for i = 1:5
    println(i)
end
```

### for với break / continue
```julia
for i = 1:10
    if i % 2 == 0
        continue    # bỏ qua số chẵn
    end
    if i > 7
        break       # dừng khi i > 7
    end
    print(i)
    print(" ")
end
# In: 1 3 5 7
```

### Vòng lặp lồng nhau
```julia
for i = 1:3
    for j = 1:3
        print(i * j)
        print(" ")
    end
    println("")
end
```

---

## 10. Hàm

### Khai báo
```julia
function tên_hàm(tham_số_1, tham_số_2)
    # thân hàm
    return giá_trị
end
```

### Ví dụ cơ bản
```julia
function add(a, b)
    return a + b
end

println(add(3, 4))    # 7
```

### Hàm đệ quy
```julia
function factorial(n)
    if n <= 1
        return 1
    end
    return n * factorial(n - 1)
end

println(factorial(6))    # 720
```

> ⚠️ **Tránh đệ quy sâu** (fib, ackermann...) — VM stack nhỏ, dễ overflow.  
> Ưu tiên dùng **vòng lặp iterative** thay cho đệ quy khi có thể.

### Hàm iterative thay cho đệ quy
```julia
# KHÔNG NÊN (đệ quy sâu, dễ overflow):
function fib_bad(n)
    if n <= 1
        return n
    end
    return fib_bad(n-1) + fib_bad(n-2)
end

# NÊN DÙNG (iterative):
function fib(n)
    if n <= 1
        return n
    end
    a = 0
    b = 1
    k = 2
    while k <= n
        c = a + b
        a = b
        b = c
        k += 1
    end
    return b
end
```

### Lưu ý về scope biến trong hàm

> ⚠️ **Biến trong hàm là global** — nếu bên trong hàm dùng biến tên `i`, nó sẽ ghi đè biến `i` bên ngoài.

```julia
# LỖI: biến i trong is_prime xung đột với for i bên ngoài
function is_prime_bad(n)
    i = 2          # ← ghi đè biến i của vòng for bên ngoài!
    while i * i <= n
        ...
        i += 1
    end
end

for i = 1:20       # i bị hàm trên làm hỏng
    is_prime_bad(i)
end

# ĐÚNG: dùng tên biến khác trong hàm
function is_prime(n)
    d = 2          # ← tên khác, không xung đột
    while d * d <= n
        if n % d == 0
            return false
        end
        d += 1
    end
    return true
end
```

---

## 11. Hàm toán học

```julia
abs(-42)          # 42   — giá trị tuyệt đối
max(3, 7)         # 7    — giá trị lớn hơn
min(3, 7)         # 3    — giá trị nhỏ hơn
sqrt(144)         # 12   — căn bậc hai (nguyên)
gcd(48, 18)       # 6    — ước chung lớn nhất
clamp(15, 0, 10)  # 10   — giới hạn trong [min, max]
clamp(-5, 0, 10)  # 0
sign(-7)          # -1   — dấu: -1 / 0 / 1
```

---

## 12. Hàm hệ thống

```julia
ticks()     # số ticks từ khi boot (>= 0)
uptime()    # thời gian uptime (>= 0)
random()    # số ngẫu nhiên >= 0
sleep(100)  # tạm dừng (đơn vị: ms hoặc ticks)
heap_free() # bytes heap còn trống
```

---

## 13. File I/O

> ⚠️ **Bắt buộc dùng đường dẫn tuyệt đối** bắt đầu bằng `/`.  
> Thư mục ghi được: `/tmp/`

### Ghi file
```julia
write_file("/tmp/hello.txt", "nội dung file")
```

### Đọc file
```julia
content = read_file("/tmp/hello.txt")
println(content)
```

### Kiểm tra file tồn tại
```julia
println(file_exists("/tmp/hello.txt"))   # true / false
```

### Nối thêm vào file
```julia
append_file("/tmp/log.txt", "dòng mới\n")
```

### Ví dụ đầy đủ
```julia
write_file("/tmp/data.txt", "line1")
println(file_exists("/tmp/data.txt"))    # true
content = read_file("/tmp/data.txt")
println(content)                         # line1
println(file_exists("/tmp/nope.txt"))    # false
```

---

## 14. In ra màn hình

```julia
println("hello")        # in + xuống dòng
print("hello ")         # in không xuống dòng
print(42)               # in số
println(true)           # in boolean
println("")             # in dòng trống
```

### Kết hợp print để in nhiều giá trị trên 1 dòng
```julia
for i = 1:5
    print(i)
    print(" ")
end
println("")    # xuống dòng cuối
# In: 1 2 3 4 5
```

---

## 15. Demo files có sẵn

Kernel tự động cài sẵn các file demo vào `/etc/julia/` lúc boot. Chạy ngay không cần build lại:

| File | Nội dung | Lệnh chạy |
|------|---------|-----------|
| `hello.jl` | Hello World cơ bản | `include("/etc/julia/hello.jl")` |
| `arithmetic.jl` | Các phép toán + `abs()` | `include("/etc/julia/arithmetic.jl")` |
| `fibonacci.jl` | Dãy Fibonacci (đệ quy) | `include("/etc/julia/fibonacci.jl")` |
| `factorial.jl` | Giai thừa 1–10 | `include("/etc/julia/factorial.jl")` |
| `fizzbuzz.jl` | FizzBuzz 1–30 | `include("/etc/julia/fizzbuzz.jl")` |
| `strings.jl` | String, nội suy, `repeat()` | `include("/etc/julia/strings.jl")` |
| `arrays.jl` | Mảng, `push!`, `sum`, `sort!` | `include("/etc/julia/arrays.jl")` |
| `forloop.jl` | For range, step, đếm ngược | `include("/etc/julia/forloop.jl")` |
| `fileio.jl` | Đọc/ghi file `/tmp/` | `include("/etc/julia/fileio.jl")` |
| `system.jl` | `uptime`, `ticks`, `random` | `include("/etc/julia/system.jl")` |
| `primes.jl` | Số nguyên tố đến 50 | `include("/etc/julia/primes.jl")` |

> ⚠️ **`fibonacci.jl`** dùng đệ quy — chỉ chạy được với `n` nhỏ (≤ 15). Với `n` lớn hơn dùng phiên bản iterative ở phần 20.

> ⚠️ **`primes.jl`** dùng biến `i` trong hàm `is_prime` — sẽ bị scope leak nếu gọi trong vòng `for i`. Chạy độc lập thì không sao.

> ✅ **Demo files được tạo tự động** mỗi khi gõ lệnh `julia` để vào REPL — `install_demo_files()` được gọi bên trong `run_repl()`.

**Lưu ý quan trọng về thời điểm tồn tại:**

| Tình huống | `/etc/julia/*.jl` có không? |
|-----------|----------------------------|
| Gõ `julia` → vào REPL → `include(...)` | ✅ Có |
| Gõ `ls /etc/julia` trước khi vào REPL | ❌ Chưa có |
| Gõ `julia /etc/julia/hello.jl` (one-shot) | ❌ Không có (one-shot không gọi `install_demo_files`) |
| Vào REPL lần 2 sau `clear` | ✅ Có (tạo lại mỗi lần vào REPL) |

Tóm lại: **phải vào REPL trước** (`julia`), sau đó mới `include("/etc/julia/...")` được.

---

## 16. Bytecode VM — kiến trúc nội bộ

Julia Tiny v0.2 chạy trên một **stack-based bytecode VM** tự viết bằng Rust. Hiểu kiến trúc này giúp debug và mở rộng ngôn ngữ.

### Tổng quan pipeline

```
Source code (.jl)
      ↓
   Lexer          (src/forthvm/lexer.rs)
      ↓  tokens
   Compiler       (src/forthvm/compiler/)
      ↓  bytecode (u32 cells)
   ForthVm        (src/forthvm/vm.rs)
      ↓
   Handlers       (src/forthvm/handlers.rs)
      ↓
   Output / Side effects
```

### Cấu trúc ForthVm

| Thành phần | Kích thước | Vai trò |
|-----------|-----------|---------|
| `memory.prog[]` | 4096 cells | Vùng chứa bytecode |
| `stack` (data) | 1024 slots | Stack tính toán |
| `callstack` | 256 slots | Stack địa chỉ return |
| `registers` | 8 regs | Thanh ghi R0–R7 |
| `strings` (pool) | dynamic | Pool chuỗi intern |
| `arrays` (pool) | dynamic | Pool mảng heap |
| `DATA_SLOTS` | 256 slots | Biến (global) |

### Định dạng instruction (32-bit cell)

```
 31      8 | 7       0
 ──────────┼──────────
   arg     │  opcode
  (24 bit) │  (8 bit)
```

Mỗi instruction là 1 `u32`: 8 bit thấp là opcode, 24 bit cao là argument (signed hoặc unsigned tùy opcode).

### Bảng opcodes đầy đủ

**Stack & hằng số**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `PUSH_INT` | 0 | Push số nguyên 24-bit lên stack |
| `PUSH_TRUE` | 1 | Push `true` |
| `PUSH_FALSE` | 2 | Push `false` |
| `PUSH_NIL` | 3 | Push `nil` |
| `PUSH_STR` | 4 | Push chuỗi (arg = string pool index) |
| `DUP` | 5 | Nhân đôi đỉnh stack |
| `DROP` | 6 | Bỏ đỉnh stack |
| `SWAP` | 7 | Đổi chỗ 2 phần tử trên đỉnh |

**Số học**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `ADD` | 10 | a + b |
| `SUB` | 11 | a - b |
| `MUL` | 12 | a * b (hoặc string repeat/concat) |
| `DIV` | 13 | a / b (chia nguyên) |
| `MOD` | 14 | a % b |
| `POW` | 15 | a ^ b (lũy thừa) |
| `NEG` | 16 | Đảo dấu (-a) |

**So sánh**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `CMP_EQ` | 20 | a == b |
| `CMP_NEQ` | 21 | a != b |
| `CMP_LT` | 22 | a < b |
| `CMP_GT` | 23 | a > b |
| `CMP_LTE` | 24 | a <= b |
| `CMP_GTE` | 25 | a >= b |

**Logic & Bitwise**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `AND` | 30 | `&&` |
| `OR` | 31 | `\|\|` |
| `NOT` | 32 | `!` |
| `BAND` | 33 | `&` |
| `BOR` | 34 | `\|` |
| `BXOR` | 35 | XOR bitwise |
| `SHL` | 36 | `<<` |
| `SHR` | 37 | `>>` |

**Điều khiển luồng**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `JMP` | 40 | Nhảy vô điều kiện đến addr |
| `JZ` | 41 | Nhảy nếu đỉnh stack falsy |
| `JNZ` | 42 | Nhảy nếu đỉnh stack truthy |
| `CALL` | 43 | Gọi hàm tại addr, push return addr |
| `RET` | 44 | Return, pop return addr |
| `HALT` | 45 | Dừng VM |

**Dữ liệu**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `LOAD` | 50 | Load biến từ slot (arg = slot index) |
| `STORE` | 51 | Store vào slot |
| `PUSH_R` | 52 | Push thanh ghi R[arg] lên stack |
| `POP_R` | 53 | Pop stack vào thanh ghi R[arg] |

**I/O**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `PRINT` | 60 | In đỉnh stack + xuống dòng |
| `PRINT_NL` | 61 | In đỉnh stack không xuống dòng |
| `READLINE` | 62 | Đọc dòng từ input, push chuỗi |

**Chuỗi**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `STR_CAT` | 70 | Ghép 2 chuỗi |
| `STR_LEN` | 71 | Độ dài chuỗi |
| `STR_ITP` | 72 | String interpolation `$var` |
| `TO_STR` | 73 | Chuyển giá trị bất kỳ → chuỗi |

**Mảng**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `ARR_NEW` | 80 | Tạo mảng rỗng |
| `ARR_PSH` | 81 | `push!(arr, val)` |
| `ARR_POP` | 82 | `pop!(arr)` |
| `ARR_GET` | 83 | `arr[index]` |
| `ARR_SET` | 84 | `arr[index] = val` |
| `ARR_LEN` | 85 | `length(arr)` |
| `ARR_LIT` | 86 | Tạo mảng literal từ N phần tử trên stack |

**Built-in & Misc**

| Opcode | Mã | Mô tả |
|--------|----|-------|
| `BUILTIN` | 90 | Gọi built-in function (arg = builtin ID) |
| `SAVE` | 95 | Lưu trạng thái VM (placeholder) |
| `RESTORE` | 96 | Khôi phục trạng thái VM (placeholder) |

### Giới hạn bộ nhớ VM

| Tài nguyên | Giới hạn |
|-----------|---------|
| Program size | 4096 instructions |
| Data stack | 1024 slots |
| Call stack | 256 frames (độ sâu đệ quy tối đa) |
| Thanh ghi | 8 (R0–R7) |
| Biến toàn cục | 256 slots |

---

## 17. Lệnh REPL

Các lệnh gõ trực tiếp trong REPL (không phải code Julia):

| Lệnh | Tác dụng |
|------|---------|
| `help` | Hiện danh sách tính năng |
| `vars` | Liệt kê tất cả biến hiện tại |
| `funcs` | Liệt kê tất cả hàm đã định nghĩa |
| `clear` | Xóa toàn bộ trạng thái VM |
| `include("/tmp/file.jl")` | Load và chạy file |
| `exit` | Thoát REPL |

---

## 18. Giới hạn & lưu ý

### ❌ Chưa hỗ trợ
| Tính năng | Trạng thái |
|-----------|-----------|
| `~x` — bitwise NOT unary | Chưa hỗ trợ |
| `$(biểu_thức)` trong chuỗi | Chỉ hỗ trợ `$tên_biến` |
| Scope biến local trong hàm | Tất cả biến là global |
| Đệ quy sâu | Dễ stack overflow |
| Số thực (float) | Chỉ có số nguyên |
| Dictionary / Map | Chưa có |
| String indexing `s[i]` | Chưa hỗ trợ |

### ✅ Quy tắc đặt tên biến an toàn
- Trong hàm, **dùng tên biến khác** với vòng lặp bên ngoài
- Tránh dùng `i`, `j`, `n` trong hàm nếu bên ngoài cũng có `for i`, `for j`

### 📁 Nhúng file vào kernel để dùng `include()`

Mở `src/fs/initramfs.rs`, thêm vào `create_default_initramfs()`:

```rust
.add_file("tmp/myfile.jl", include_bytes!("../../myfile.jl"))
```

Đặt `myfile.jl` cạnh `Cargo.toml` (thư mục gốc project), rồi:

```
cargo build
```

Sau đó trong REPL:
```
jl> include("/tmp/myfile.jl")
```

---

## 19. Tính năng khác

### println / print nhiều tham số

`println` và `print` hỗ trợ **nhiều tham số cách nhau bằng dấu phẩy** — in lần lượt không có dấu cách:

```julia
println("x = ", x, " y = ", y)
print("fib(", 5, ") = ", fib(5))
println("")
```

Tương đương nối chuỗi thủ công nhưng gọn hơn nhiều.

---

### Nối chuỗi bằng `*`

```julia
greeting = "Hello" * " " * "World!"
println(greeting)    # Hello World!
```

Lặp chuỗi bằng `*` với số nguyên:

```julia
println(repeat("=-", 15))    # =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
println("-" * 20)             # ---- (20 dấu gạch)
```

> Lưu ý: `repeat(s, n)` và `s * n` cho kết quả giống nhau.

---

### For loop với step (bước nhảy)

Cú pháp `for i = start:step:end`:

```julia
# Số chẵn từ 0 đến 20
for i = 0:2:20
    print(i, " ")
end
println("")
# 0 2 4 6 8 10 12 14 16 18 20
```

### For loop đếm ngược

Step âm để đếm ngược:

```julia
for i = 10:-1:1
    print(i, " ")
end
println("Go!")
# 10 9 8 7 6 5 4 3 2 1 Go!
```

---

### repeat() — lặp chuỗi

```julia
repeat("ab", 4)    # abababab
repeat("-", 20)    # --------------------
```

---

### Vị trí file trong VFS

| Thư mục | Nội dung | Ghi/Đọc |
|---------|---------|---------|
| `/etc/julia/` | Demo files cài sẵn lúc boot | Chỉ đọc |
| `/tmp/` | Thư mục ghi tạm thời | Đọc + Ghi |
| `/etc/` | Config files hệ thống | Chỉ đọc |
| `/bin/` | Binary scripts | Chỉ đọc |

---

## 20. Ví dụ đầy đủ

### Kiểm tra số nguyên tố
```julia
function is_prime(n)
    if n < 2
        return false
    end
    d = 2
    while d * d <= n
        if n % d == 0
            return false
        end
        d += 1
    end
    return true
end

for i = 1:50
    if is_prime(i)
        print(i)
        print(" ")
    end
end
println("")
# 2 3 5 7 11 13 17 19 23 29 31 37 41 43 47
```

### Sắp xếp mảng (Bubble Sort)
```julia
function bubble_sort(arr)
    n = length(arr)
    i = 1
    while i <= n
        j = 1
        while j <= n - i
            if arr[j] > arr[j+1]
                tmp = arr[j]
                arr[j] = arr[j+1]
                arr[j+1] = tmp
            end
            j += 1
        end
        i += 1
    end
    return arr
end

data = [5, 2, 8, 1, 9, 3]
bubble_sort(data)
for k = 1:length(data)
    print(data[k])
    print(" ")
end
println("")
# 1 2 3 5 8 9
```

### Fibonacci iterative
```julia
function fib(n)
    if n <= 1
        return n
    end
    a = 0
    b = 1
    k = 2
    while k <= n
        c = a + b
        a = b
        b = c
        k += 1
    end
    return b
end

for i = 0:10
    print(fib(i))
    print(" ")
end
println("")
# 0 1 1 2 3 5 8 13 21 34 55
```

### Đọc/ghi file
```julia
write_file("/tmp/note.txt", "MyKernel is awesome")
if file_exists("/tmp/note.txt")
    content = read_file("/tmp/note.txt")
    println(content)
end
```

---

*Julia Tiny v0.2 — MyKernel · Tài liệu dựa trên test thực tế 
