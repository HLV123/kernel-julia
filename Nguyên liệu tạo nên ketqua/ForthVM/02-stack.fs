\ ============================================================
\ 02-stack.fs -- Ngăn xếp máy ảo
\ Quy ước Empty-Ascending: vsp trỏ vào ô tiếp theo sẽ ghi
\   PUSH: ghi vào stack[vsp], rồi tăng vsp
\   POP:  giảm vsp, rồi đọc từ stack[vsp]
\ ============================================================

\ Đẩy giá trị lên đỉnh ngăn xếp
: vm-push ( x -- )
  vsp @ STACK-SIZE >= if ." LOI: tran ngan xep" cr abort then
  vsp @ cells stack + !
  vsp @ 1+ vsp ! ;

\ Lấy giá trị từ đỉnh ngăn xếp
: vm-pop ( -- x )
  vsp @ 0= if ." LOI: ngan xep rong" cr abort then
  vsp @ 1- vsp !
  vsp @ cells stack + @ ;

\ Xem giá trị trên đỉnh mà không lấy ra
: vm-peek ( -- x )
  vsp @ 0= if ." LOI: ngan xep rong" cr abort then
  vsp @ 1- cells stack + @ ;
