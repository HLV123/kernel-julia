\ ============================================================
\ 03-registers.fs -- Thanh ghi R0–R7
\ 8 thanh ghi đa dụng, truy cập qua chỉ số 0–7
\ ============================================================

\ Đọc giá trị thanh ghi (reg → val)
: r@ ( reg -- val )
  dup 0 REG-COUNT 1- in-range? 0= if ." LOI: thanh ghi khong hop le" cr abort then
  cells reg + @ ;

\ Ghi giá trị vào thanh ghi (val reg →)
: r! ( val reg -- )
  dup 0 REG-COUNT 1- in-range? 0= if ." LOI: thanh ghi khong hop le" cr abort then
  cells reg + ! ;
