\ ============================================================
\ 05-callstack.fs -- Ngăn xếp lời gọi hàm
\ Lưu địa chỉ trả về khi CALL, khôi phục khi RET
\ ============================================================

\ Đẩy địa chỉ trả về lên ngăn xếp gọi
: cs-push ( x -- )
  vcsp @ CSTACK-SIZE >= if ." LOI: tran ngan xep goi" cr abort then
  vcsp @ cells callstack + !
  vcsp @ 1+ vcsp ! ;

\ Lấy địa chỉ trả về từ ngăn xếp gọi
: cs-pop ( -- x )
  vcsp @ 0= if ." LOI: ngan xep goi rong" cr abort then
  vcsp @ 1- vcsp !
  vcsp @ cells callstack + @ ;
