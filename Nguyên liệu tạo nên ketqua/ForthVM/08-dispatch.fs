\ ============================================================
\ 08-dispatch.fs -- Phân phối lệnh & vòng lặp thực thi
\ Đọc mã lệnh → gọi handler tương ứng → lặp lại
\ ============================================================

\ Bảng phân phối: nhận mã lệnh, gọi handler đúng
: dispatch ( opcode -- )
  dup OP_PUSH       = if drop op-push       exit then
  dup OP_ADD        = if drop op-add        exit then
  dup OP_SUB        = if drop op-sub        exit then
  dup OP_MUL        = if drop op-mul        exit then
  dup OP_PUSH_R     = if drop op-push-r     exit then
  dup OP_POP_R      = if drop op-pop-r      exit then
  dup OP_PRINT      = if drop op-print      exit then
  dup OP_JMP        = if drop op-jmp        exit then
  dup OP_JZ         = if drop op-jz         exit then
  dup OP_JGT        = if drop op-jgt        exit then
  dup OP_CALL       = if drop op-call       exit then
  dup OP_RET        = if drop op-ret        exit then
  dup OP_HALT       = if drop op-halt       exit then
  dup OP_DUP        = if drop op-dup        exit then
  dup OP_DROP       = if drop op-drop       exit then
  dup OP_SWAP       = if drop op-swap       exit then
  dup OP_LOAD_DATA  = if drop op-load-data  exit then
  dup OP_STORE_DATA = if drop op-store-data exit then
  dup OP_ALLOC      = if drop op-alloc      exit then
  dup OP_FREE       = if drop op-free       exit then
  dup OP_HEAP_LOAD  = if drop op-heap-load  exit then
  dup OP_HEAP_STORE = if drop op-heap-store exit then
  dup OP_SAVE       = if drop op-save       exit then
  dup OP_RESTORE    = if drop op-restore    exit then
  dup OP_CMP_EQ     = if drop op-cmp-eq     exit then
  dup OP_CMP_GT     = if drop op-cmp-gt     exit then
  ." LOI: ma lenh khong hop le " . cr abort ;

\ Chạy liên tục: nạp lệnh → thực thi → lặp cho đến HALT
: vm-run ( -- )
  1 running !
  begin running @ while
    fetch dispatch
  repeat ;

\ Chạy đúng 1 lệnh (dùng cho debugger)
: vm-step ( -- )
  running @ if fetch dispatch then ;
