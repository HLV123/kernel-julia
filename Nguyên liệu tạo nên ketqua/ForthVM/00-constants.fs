\ ============================================================
\ 00-constants.fs -- Hằng số hệ thống
\ Định nghĩa kích thước bộ nhớ, mã lệnh, phân vùng
\ ============================================================

\ --- Kích thước bộ nhớ ---
1024 constant PROG-SIZE      \ Vùng chương trình (cells)
1024 constant STACK-SIZE     \ Ngăn xếp dữ liệu (cells)
256  constant CSTACK-SIZE    \ Ngăn xếp lời gọi hàm (cells)
8    constant REG-COUNT      \ Số thanh ghi R0–R7

\ --- Phân vùng bộ nhớ chương trình ---
\ Mảng program[] được chia thành 4 vùng:
\   [0..255]    Code   – chứa bytecode
\   [256..383]  Data   – biến toàn cục
\   [384..511]  Stack  – dự phòng
\   [512..1023] Heap   – bộ nhớ động (arena)
0    constant SEG_CODE_BASE
256  constant SEG_DATA_BASE
384  constant SEG_STACK_BASE
512  constant SEG_HEAP_BASE
255  constant SEG_CODE_END
383  constant SEG_DATA_END
511  constant SEG_STACK_END
1023 constant SEG_HEAP_END

\ --- Mã lệnh (Opcode) ---
\ Mỗi lệnh mã hoá thành 1 cell: (arg << 8) | opcode
\ 8 bit thấp = mã lệnh, các bit còn lại = tham số
0  constant OP_PUSH         \ Đẩy giá trị lên ngăn xếp
1  constant OP_ADD          \ Cộng 2 giá trị trên đỉnh
2  constant OP_SUB          \ Trừ: phần tử dưới − phần tử trên
3  constant OP_MUL          \ Nhân 2 giá trị trên đỉnh
4  constant OP_PUSH_R       \ Đẩy giá trị thanh ghi lên ngăn xếp
5  constant OP_POP_R        \ Lấy từ ngăn xếp vào thanh ghi
6  constant OP_PRINT        \ In và xoá giá trị trên đỉnh
7  constant OP_JMP          \ Nhảy vô điều kiện
8  constant OP_JZ           \ Nhảy nếu giá trị = 0
9  constant OP_CALL         \ Gọi chương trình con
10 constant OP_RET          \ Trở về từ chương trình con
11 constant OP_HALT         \ Dừng máy ảo
12 constant OP_DUP          \ Nhân đôi giá trị trên đỉnh
13 constant OP_DROP         \ Xoá giá trị trên đỉnh
14 constant OP_SWAP         \ Đổi chỗ 2 giá trị trên đỉnh
15 constant OP_LOAD_DATA    \ Đọc từ vùng dữ liệu
16 constant OP_STORE_DATA   \ Ghi vào vùng dữ liệu
17 constant OP_ALLOC        \ Cấp phát bộ nhớ heap
18 constant OP_FREE         \ Giải phóng bộ nhớ heap
19 constant OP_HEAP_LOAD    \ Đọc từ heap
20 constant OP_HEAP_STORE   \ Ghi vào heap
21 constant OP_JGT          \ Nhảy nếu > 0
22 constant OP_SAVE         \ In trạng thái VM ra màn hình
23 constant OP_RESTORE      \ Khôi phục trạng thái (tạm tắt)
24 constant OP_CMP_EQ       \ So sánh bằng: đẩy 1 nếu bằng
25 constant OP_CMP_GT       \ So sánh lớn hơn: đẩy 1 nếu >

\ --- Hàm trợ giúp ---
: in-range? ( n lo hi -- cờ )
  rot dup rot <= -rot <= and ;

\ Đóng gói tham số và mã lệnh thành 1 cell
: pack ( arg opcode -- packed )
  swap 8 lshift or ;

\ So sánh 2 chuỗi (dùng cho assembler)
: str= ( a1 l1 a2 l2 -- cờ ) compare 0= ;
