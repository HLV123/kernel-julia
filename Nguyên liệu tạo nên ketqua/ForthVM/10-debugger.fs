\ ============================================================
\ 10-debugger.fs -- Trình gỡ lỗi tương tác (Phase 4)
\ Hiển thị bytecode, thanh ghi, ngăn xếp; bước từng lệnh
\ ============================================================

\ Tìm vị trí HALT đầu tiên trong vùng code
: find-prog-end ( -- )
  SEG_CODE_END 0 do
    i prog@ 255 and OP_HALT = if i 1+ prog-end ! unloop exit then
  loop
  SEG_CODE_END 1+ prog-end ! ;

\ --- Hiển thị trạng thái VM ---
: .vm-state ( -- )
  cr
  ." PC: " vpc @ . ."  SP: " vsp @ . ."  HEAP: " heap-ptr @ . cr
  ." THANH GHI:" cr
  ."   R0: " 0 r@ . ."   R4: " 4 r@ . cr
  ."   R1: " 1 r@ . ."   R5: " 5 r@ . cr
  ."   R2: " 2 r@ . ."   R6: " 6 r@ . cr
  ."   R3: " 3 r@ . ."   R7: " 7 r@ . cr
  ." NGAN XEP:" cr
  vsp @ 0= if ."   [ rong ]" cr else
    vsp @ 0 do ."   [" i . ." ] " i cells stack + @ . cr loop
  then ;

\ --- Các bảng hiển thị debugger ---
: .panel-bytecode ( -- )
  ." BYTECODE" cr
  0 .cur !
  begin .cur @ prog-end @ < while
    vpc @ .cur @ = if ." > " else ."   " then
    .cur @ 3 .r ." : "
    .cur @ disasm-one .cur ! cr
  repeat ;

: .panel-regs ( -- )
  ." THANH GHI" cr
  ."   R0: " 0 r@ 5 .r ."   R4: " 4 r@ 5 .r cr
  ."   R1: " 1 r@ 5 .r ."   R5: " 5 r@ 5 .r cr
  ."   R2: " 2 r@ 5 .r ."   R6: " 6 r@ 5 .r cr
  ."   R3: " 3 r@ 5 .r ."   R7: " 7 r@ 5 .r cr ;

: .panel-stack ( -- )
  ." NGAN XEP" cr
  vsp @ 0= if ."   [ rong ]" cr else
    vsp @ 0 do
      vsp @ i - 1- cells stack + @
      i 0= if ."   dinh-> " else ."          " then
      . cr
    loop
  then ;

: .panel-sep ." ======================================================" cr ;
: .panel-div ." ------------------------------------------------------" cr ;

\ Bảng debugger hoàn chỉnh
: .debugger-panel ( -- )
  .panel-sep .panel-bytecode .panel-div
  .panel-regs .panel-div .panel-stack
  ."   PC: " vpc @ . ."  SP: " vsp @ . ."  HEAP: " heap-ptr @ . cr
  .panel-sep ;

\ Đọc 1 ký tự lệnh từ bàn phím
: read-cmd ( -- char )
  pad 80 accept drop pad c@ ;

\ Vòng lặp debugger chính
\ [n] bước tiếp, [r] chạy hết, [q] thoát
: debugger-loop ( -- )
  find-prog-end
  1 running !
  begin
    .debugger-panel
    ."   [n] buoc tiep   [r] chay het   [q] thoat" cr ." > "
    read-cmd
    dup [char] n = if drop vm-step     else
    dup [char] r = if drop vm-run      else
    dup [char] q = if drop 0 running ! else
    drop then then then
    running @ 0= vpc @ prog-end @ >= or
  until
  .debugger-panel
  cr ." [May ao da dung]" cr ;
