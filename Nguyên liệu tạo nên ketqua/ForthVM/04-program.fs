\ ============================================================
\ 04-program.fs -- Bộ nhớ chương trình & nạp lệnh
\ Đọc/ghi bytecode, nạp lệnh tiếp theo (fetch)
\ ============================================================

\ Ghi giá trị vào vị trí addr trong program[]
: prog! ( val addr -- )
  dup 0 PROG-SIZE 1- in-range? 0= if ." LOI: ghi ngoai vung" cr abort then
  cells program + ! ;

\ Đọc giá trị tại vị trí addr trong program[]
: prog@ ( addr -- val )
  dup 0 PROG-SIZE 1- in-range? 0= if ." LOI: doc ngoai vung" cr abort then
  cells program + @ ;

\ Biến lưu tham số của lệnh hiện tại
variable arg

\ Nạp lệnh tiếp theo tại vpc, tách mã lệnh và tham số
\ Mỗi cell đóng gói: (arg << 8) | opcode
\   - 8 bit thấp → mã lệnh (opcode)
\   - Các bit cao → tham số (arg)
: fetch ( -- opcode )
  vpc @ prog@           \ đọc cell tại vpc
  vpc @ 1+ vpc !        \ tăng vpc
  dup 8 rshift arg !    \ tách tham số → lưu vào arg
  255 and ;             \ tách mã lệnh → trả về
