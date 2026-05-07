\ ============================================================
\ 07-handlers.fs -- Bộ xử lý lệnh (Opcode Handlers)
\ Mỗi lệnh máy ảo tương ứng với một từ Forth
\ ============================================================

\ --- Phase 1: Phép tính cơ bản ---
: op-push       ( -- ) arg @ vm-push ;                          \ Đẩy hằng số
: op-add        ( -- ) vm-pop vm-pop + vm-push ;                \ Cộng
: op-sub        ( -- ) vm-pop vm-pop swap - vm-push ;           \ Trừ
: op-mul        ( -- ) vm-pop vm-pop * vm-push ;                \ Nhân
: op-push-r     ( -- ) arg @ r@ vm-push ;                      \ Đẩy thanh ghi
: op-pop-r      ( -- ) arg @ vm-pop swap r! ;                  \ Lưu vào thanh ghi
: op-print      ( -- ) vm-pop . cr ;                           \ In giá trị
: op-halt       ( -- ) 0 running ! ;                           \ Dừng máy ảo
: op-dup        ( -- ) vm-peek vm-push ;                       \ Nhân đôi đỉnh
: op-drop       ( -- ) vm-pop drop ;                           \ Xoá đỉnh
: op-swap       ( -- ) vm-pop vm-pop swap vm-push vm-push ;    \ Đổi 2 đỉnh

\ --- Phase 2: Điều khiển luồng ---
: op-jmp        ( -- ) arg @ vpc ! ;                           \ Nhảy vô điều kiện
: op-jz         ( -- ) arg @ vm-pop 0= if vpc ! else drop then ; \ Nhảy nếu = 0
: op-jgt        ( -- ) arg @ vm-pop 0 > if vpc ! else drop then ; \ Nhảy nếu > 0

\ --- Phase 3: Gọi hàm ---
: op-call       ( -- ) arg @ vpc @ cs-push vpc ! ;             \ Gọi: lưu PC, nhảy
: op-ret        ( -- ) cs-pop vpc ! ;                          \ Trả về: khôi phục PC

\ --- Phase 5: Bộ nhớ ---
: op-load-data  ( -- ) arg @ data@ vm-push ;                   \ Đọc biến toàn cục
: op-store-data ( -- ) arg @ vm-pop swap data! ;               \ Ghi biến toàn cục
: op-alloc      ( -- ) arg @ heap-alloc vm-push ;              \ Cấp phát heap
: op-free       ( -- ) vm-pop heap-free ;                      \ Giải phóng heap
: op-heap-load  ( -- ) vm-pop prog@ vm-push ;                  \ Đọc từ heap
: op-heap-store ( -- ) vm-pop vm-pop swap prog! ;              \ Ghi vào heap

\ --- Phase 7: Trạng thái ---
: op-save ( -- )
  cr ." === TRANG THAI VM ===" cr
  ." PC: " vpc @ . ."  SP: " vsp @ . ."  CSP: " vcsp @ . ."  HEAP: " heap-ptr @ . cr
  ." Thanh ghi:" 8 0 do ."  R" i . ." =" i r@ . loop cr
  ." Ngan xep (" vsp @ . ." phan tu):" cr
  vsp @ 0 do i . ." : " i cells stack + @ . cr loop
  ." === KET THUC ===" cr ;

: op-restore ( -- )
  cr ." Khoi phuc tu file chua duoc ho tro." cr ;

\ --- Phase 8: So sánh ---
: op-cmp-eq ( -- ) vm-pop vm-pop = if 1 vm-push else 0 vm-push then ;   \ Bằng
: op-cmp-gt ( -- ) vm-pop vm-pop swap > if 1 vm-push else 0 vm-push then ; \ Lớn hơn
