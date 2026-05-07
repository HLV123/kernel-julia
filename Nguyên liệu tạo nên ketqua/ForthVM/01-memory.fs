\ ============================================================
\ 01-memory.fs -- Bố trí bộ nhớ
\ Tạo 4 vùng nhớ chính và các biến trạng thái
\ ============================================================

\ --- 4 vùng nhớ chính của máy ảo ---
create program   PROG-SIZE  cells allot   \ Bytecode + dữ liệu + heap
create stack     STACK-SIZE cells allot   \ Ngăn xếp dữ liệu riêng của VM
create callstack CSTACK-SIZE cells allot  \ Ngăn xếp lưu địa chỉ trả về
create reg       REG-COUNT  cells allot   \ 8 thanh ghi đa dụng

\ --- Các biến trạng thái ---
variable vpc       \ Con trỏ lệnh (Program Counter)
variable vsp       \ Con trỏ đỉnh ngăn xếp (Stack Pointer)
variable vcsp      \ Con trỏ ngăn xếp lời gọi (Call Stack Pointer)
variable running   \ Cờ chạy: 1 = đang chạy, 0 = đã dừng
variable prog-end  \ Vị trí kết thúc chương trình (cho debugger)
variable .cur      \ Biến tạm cho trình dịch ngược
variable heap-ptr  \ Con trỏ heap – vị trí tự do tiếp theo

\ --- Khởi tạo heap ---
: heap-init  ( -- ) SEG_HEAP_BASE heap-ptr ! ;
: heap-reset ( -- ) SEG_HEAP_BASE heap-ptr ! ;

\ --- Khởi tạo toàn bộ máy ảo ---
\ Đặt tất cả thanh ghi, con trỏ về 0, reset heap
: vm-init ( -- )
  0 vpc ! 0 vsp ! 0 vcsp ! 0 running !
  REG-COUNT 0 do 0 i cells reg + ! loop
  heap-init ;
