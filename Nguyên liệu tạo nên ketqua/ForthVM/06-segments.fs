\ ============================================================
\ 06-segments.fs -- Phân vùng bộ nhớ
\ Truy cập vùng Data (biến toàn cục) và Heap (bộ nhớ động)
\ ============================================================

\ --- Vùng Data: biến toàn cục ---
\ Slot 0 → program[256], slot 1 → program[257], ...
: data@ ( slot -- val ) SEG_DATA_BASE + prog@ ;
: data! ( val slot -- ) SEG_DATA_BASE + prog! ;

\ --- Vùng Heap: cấp phát động kiểu arena ---
\ Cấp phát n cells liên tiếp, trả về địa chỉ đầu
: heap-alloc ( n -- addr )
  heap-ptr @ over + SEG_HEAP_END 1+ > if
    ." LOI: heap het bo nho" cr abort
  then
  heap-ptr @ swap heap-ptr +! ;

\ Giải phóng – đặt lại con trỏ heap về addr
\ (chỉ an toàn nếu giải phóng theo thứ tự ngược)
: heap-free ( addr -- )
  dup SEG_HEAP_BASE < if
    ." LOI: dia chi heap khong hop le" cr abort
  then
  heap-ptr ! ;
