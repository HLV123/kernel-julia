\ ============================================================
\ 11-assembler.fs -- Trình hợp dịch văn bản (Phase 6)
\ Chuyển chuỗi assembly "PUSH 5 ADD HALT" → bytecode
\ ============================================================

\ --- Bộ phát bytecode ---
variable asm-ptr      \ Vị trí ghi bytecode tiếp theo
variable asm-start    \ Vị trí bắt đầu đoạn code

: asm-begin ( -- ) vm-init 0 asm-ptr ! 0 asm-start ! ;
: asm-emit ( n -- ) asm-ptr @ prog! asm-ptr @ 1+ asm-ptr ! ;
: asm-emit-packed ( arg opcode -- ) pack asm-emit ;
: asm-count ( -- n ) asm-ptr @ asm-start @ - ;

\ --- Bộ đệm xây dựng lệnh Forth ---
1024 constant ASM-BUF-SIZE
create asm-buf ASM-BUF-SIZE allot
variable asm-buf-len

: asm-buf-reset ( -- ) 0 asm-buf-len ! ;
: asm-buf-add-char ( c -- )
  asm-buf-len @ ASM-BUF-SIZE 1- < if
    asm-buf asm-buf-len @ + c! asm-buf-len @ 1+ asm-buf-len !
  else drop then ;

variable asm-bw-src  variable asm-bw-n
: asm-buf-add-str ( addr len -- )
  asm-bw-n ! asm-bw-src !
  asm-bw-n @ 0 do asm-bw-src @ i + c@ asm-buf-add-char loop ;

\ --- Bộ quét mã nguồn assembly ---
variable asm-src-a  variable asm-src-l
: asm-src-more ( -- f ) asm-src-l @ 0> ;
: asm-src-char ( -- c ) asm-src-a @ c@ ;
: asm-src-adv  ( -- ) asm-src-a @ 1+ asm-src-a ! asm-src-l @ 1- asm-src-l ! ;
: asm-skip-ws ( -- )
  begin asm-src-more asm-src-char bl <= and while asm-src-adv repeat ;

256 constant TOK-SIZE
create asm-tok TOK-SIZE allot
variable asm-tok-len

: asm-read-tok ( -- addr len )
  asm-skip-ws  0 asm-tok-len !
  begin asm-src-more asm-src-char bl > and while
    asm-src-char asm-tok asm-tok-len @ + c!
    asm-tok-len @ 1+ asm-tok-len ! asm-src-adv
  repeat
  asm-tok asm-tok-len @ ;

\ --- Bảng lệnh có tham số ---
: asm-has-arg? ( addr len -- cờ )
  2dup s" PUSH"       str= if 2drop true exit then
  2dup s" PUSH_R"     str= if 2drop true exit then
  2dup s" POP_R"      str= if 2drop true exit then
  2dup s" JMP"        str= if 2drop true exit then
  2dup s" JZ"         str= if 2drop true exit then
  2dup s" JGT"        str= if 2drop true exit then
  2dup s" CALL"       str= if 2drop true exit then
  2dup s" LOAD_DATA"  str= if 2drop true exit then
  2dup s" STORE_DATA" str= if 2drop true exit then
  2dup s" ALLOC"      str= if 2drop true exit then
  2dup s" SAVE"       str= if 2drop true exit then
  2dup s" RESTORE"    str= if 2drop true exit then
  2drop false ;

: asm-is-num? ( addr len -- cờ )
  drop c@ dup [char] 0 >= over [char] 9 <= and swap [char] - = or ;

\ --- Xây dựng chuỗi Forth từ assembly ---
256 constant MN-SIZE
create asm-mn MN-SIZE allot
variable asm-mn-len

: asm-build-eval ( src-addr src-len -- )
  asm-buf-reset asm-src-l ! asm-src-a !
  begin asm-src-more while
    asm-read-tok asm-tok-len @ 0= if 2drop exit then
    2dup asm-is-num? if
      asm-buf-add-str bl asm-buf-add-char
    else
      2dup asm-has-arg? if
        dup asm-mn-len !
        asm-mn-len @ 0 do asm-tok i + c@ asm-mn i + c! loop 2drop
        asm-read-tok asm-buf-add-str bl asm-buf-add-char
        s" A." asm-buf-add-str asm-mn asm-mn-len @ asm-buf-add-str bl asm-buf-add-char
      else
        s" A." asm-buf-add-str asm-buf-add-str bl asm-buf-add-char
      then
    then
  repeat ;

\ Biên dịch chuỗi assembly → bytecode
: asm-eval ( src-addr src-len -- )
  asm-build-eval asm-buf asm-buf-len @ evaluate ;

\ --- Các từ assembly (gọi bởi asm-eval) ---
: A.PUSH   ( n -- ) OP_PUSH   asm-emit-packed ;
: A.PUSH_R ( n -- ) OP_PUSH_R asm-emit-packed ;
: A.POP_R  ( n -- ) OP_POP_R  asm-emit-packed ;
: A.JMP    ( n -- ) OP_JMP    asm-emit-packed ;
: A.JZ     ( n -- ) OP_JZ     asm-emit-packed ;
: A.JGT    ( n -- ) OP_JGT    asm-emit-packed ;
: A.CALL   ( n -- ) OP_CALL   asm-emit-packed ;
: A.LOAD_DATA  ( n -- ) OP_LOAD_DATA  asm-emit-packed ;
: A.STORE_DATA ( n -- ) OP_STORE_DATA asm-emit-packed ;
: A.ALLOC  ( n -- ) OP_ALLOC  asm-emit-packed ;
: A.SAVE   ( n -- ) OP_SAVE   asm-emit-packed ;
: A.RESTORE ( n -- ) OP_RESTORE asm-emit-packed ;
: A.ADD    ( -- ) 0 OP_ADD    asm-emit-packed ;
: A.SUB    ( -- ) 0 OP_SUB    asm-emit-packed ;
: A.MUL    ( -- ) 0 OP_MUL    asm-emit-packed ;
: A.PRINT  ( -- ) 0 OP_PRINT  asm-emit-packed ;
: A.RET    ( -- ) 0 OP_RET    asm-emit-packed ;
: A.HALT   ( -- ) 0 OP_HALT   asm-emit-packed ;
: A.DUP    ( -- ) 0 OP_DUP    asm-emit-packed ;
: A.DROP   ( -- ) 0 OP_DROP   asm-emit-packed ;
: A.SWAP   ( -- ) 0 OP_SWAP   asm-emit-packed ;
: A.FREE   ( -- ) 0 OP_FREE   asm-emit-packed ;
: A.HEAP_LOAD  ( -- ) 0 OP_HEAP_LOAD  asm-emit-packed ;
: A.HEAP_STORE ( -- ) 0 OP_HEAP_STORE asm-emit-packed ;
: A.CMP_EQ ( -- ) 0 OP_CMP_EQ asm-emit-packed ;
: A.CMP_GT ( -- ) 0 OP_CMP_GT asm-emit-packed ;
