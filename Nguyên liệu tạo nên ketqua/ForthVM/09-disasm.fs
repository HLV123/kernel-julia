\ ============================================================
\ 09-disasm.fs -- Trình dịch ngược (Disassembler)
\ Chuyển bytecode thành văn bản dễ đọc
\ ============================================================

\ Tiến con trỏ dịch ngược 1 bước
: disasm-advance ( addr -- addr+1 ) 1+ ;

\ Dịch ngược 1 lệnh tại addr, trả về addr tiếp theo
: disasm-one ( addr -- next-addr )
  dup prog@
  dup 255 and swap 8 rshift swap  \ tách: ( addr arg opcode )
  dup OP_PUSH       = if drop ." PUSH      " 4 .r 1+ exit then
  dup OP_ADD        = if drop drop ." ADD       " 1+ exit then
  dup OP_SUB        = if drop drop ." SUB       " 1+ exit then
  dup OP_MUL        = if drop drop ." MUL       " 1+ exit then
  dup OP_PUSH_R     = if drop ." PUSH_R    " 4 .r 1+ exit then
  dup OP_POP_R      = if drop ." POP_R     " 4 .r 1+ exit then
  dup OP_PRINT      = if drop drop ." PRINT     " 1+ exit then
  dup OP_JMP        = if drop ." JMP       " 4 .r 1+ exit then
  dup OP_JZ         = if drop ." JZ        " 4 .r 1+ exit then
  dup OP_JGT        = if drop ." JGT       " 4 .r 1+ exit then
  dup OP_CALL       = if drop ." CALL      " 4 .r 1+ exit then
  dup OP_RET        = if drop drop ." RET       " 1+ exit then
  dup OP_HALT       = if drop drop ." HALT      " 1+ exit then
  dup OP_DUP        = if drop drop ." DUP       " 1+ exit then
  dup OP_DROP       = if drop drop ." DROP      " 1+ exit then
  dup OP_SWAP       = if drop drop ." SWAP      " 1+ exit then
  dup OP_LOAD_DATA  = if drop ." LOAD_DATA " 4 .r 1+ exit then
  dup OP_STORE_DATA = if drop ." STORE_DATA" 4 .r 1+ exit then
  dup OP_ALLOC      = if drop ." ALLOC     " 4 .r 1+ exit then
  dup OP_FREE       = if drop drop ." FREE      " 1+ exit then
  dup OP_HEAP_LOAD  = if drop drop ." HEAP_LOAD " 1+ exit then
  dup OP_HEAP_STORE = if drop drop ." HEAP_STORE" 1+ exit then
  dup OP_SAVE       = if drop drop ." SAVE      " 1+ exit then
  dup OP_RESTORE    = if drop drop ." RESTORE   " 1+ exit then
  dup OP_CMP_EQ     = if drop drop ." CMP_EQ    " 1+ exit then
  dup OP_CMP_GT     = if drop drop ." CMP_GT    " 1+ exit then
  drop drop ." ???       " 1+ ;

\ Dịch ngược từ địa chỉ 0 đến end-addr
: disasm ( end-addr -- )
  0 .cur !
  begin .cur @ over < while
    .cur @ 3 .r ." : "
    .cur @ disasm-one .cur !
    cr
  repeat drop ;
