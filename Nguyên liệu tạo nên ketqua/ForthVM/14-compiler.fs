\ ============================================================
\ 14-compiler.fs -- Trình biên dịch Julia (Phase 8)
\ Phân tích cú pháp đệ quy → phát bytecode trực tiếp
\ ============================================================

\ --- Khai báo trước (đệ quy chéo) ---
defer jl-parse-expr    \ Biểu thức
defer jl-parse-stmt    \ Câu lệnh

\ --- Hàm hỗ trợ nhảy (backpatch) ---
: jl-here ( -- addr ) asm-ptr @ ;
: jl-emit-jz-ph  ( -- patch ) jl-here 0 OP_JZ  asm-emit-packed ;
: jl-emit-jmp-ph ( -- patch ) jl-here 0 OP_JMP asm-emit-packed ;
: jl-patch ( target patch-addr -- )
  dup prog@ 255 and rot 8 lshift or swap prog! ;

\ --- Bộ đệm tham số hàm ---
8 constant MAX-PARAMS
create param-slots MAX-PARAMS cells allot
variable param-count

\ --- Bỏ qua dòng trống ---
: jl-skip-nl ( -- )
  begin tok-type @ TK_NEWLINE = while jl-next repeat ;

\ --- Phân tích khối lệnh (dừng tại end / else / eof) ---
: jl-parse-block ( -- )
  begin jl-skip-nl
    tok-type @ TK_END <> tok-type @ TK_ELSE <> and tok-type @ TK_EOF <> and
  while jl-parse-stmt repeat ;

\ --- println(biểu_thức) ---
: jl-parse-println ( -- )
  jl-next
  tok-type @ TK_LPAREN <> if ." LOI: can ( sau println" cr abort then
  jl-next jl-parse-expr
  tok-type @ TK_RPAREN <> if ." LOI: can ) sau println" cr abort then
  jl-next 0 OP_PRINT asm-emit-packed ;

\ --- if điều_kiện ... else ... end ---
: jl-parse-if ( -- )
  jl-next jl-parse-expr jl-emit-jz-ph jl-skip-nl jl-parse-block
  tok-type @ TK_ELSE = if
    jl-next jl-emit-jmp-ph swap jl-here swap jl-patch
    jl-skip-nl jl-parse-block jl-here swap jl-patch
  else jl-here swap jl-patch then
  tok-type @ TK_END <> if ." LOI: can 'end'" cr abort then jl-next ;

\ --- while điều_kiện ... end ---
: jl-parse-while ( -- )
  jl-next jl-here jl-parse-expr jl-emit-jz-ph jl-skip-nl jl-parse-block
  swap OP_JMP asm-emit-packed jl-here swap jl-patch
  tok-type @ TK_END <> if ." LOI: can 'end'" cr abort then jl-next ;

\ --- return biểu_thức ---
: jl-parse-return ( -- )
  jl-next jl-parse-expr 0 OP_RET asm-emit-packed ;

\ --- Gọi hàm: tra tên → đẩy tham số → CALL ---
: jl-emit-call ( -- )
  saved-name saved-name-len @ func-find
  0= if ." LOI: ham khong ton tai '" saved-name saved-name-len @ type ." '" cr abort then
  drop >r jl-next
  begin tok-type @ TK_RPAREN <> while
    jl-parse-expr tok-type @ TK_COMMA = if jl-next then
  repeat jl-next r> OP_CALL asm-emit-packed ;

\ --- Định nghĩa hàm ---
variable func-jmp-patch  variable func-entry-addr

: jl-parse-params ( -- )
  0 param-count !
  tok-type @ TK_LPAREN <> if ." LOI: can (" cr abort then jl-next
  begin tok-type @ TK_RPAREN <> while
    tok-type @ TK_IDENT <> if ." LOI: can ten tham so" cr abort then
    tok-id tok-id-len @ sym-find-or-add
    param-count @ cells param-slots + ! param-count @ 1+ param-count ! jl-next
    tok-type @ TK_COMMA = if jl-next then
  repeat jl-next ;

: jl-emit-prologue ( -- )
  param-count @ 0 do
    param-count @ i - 1- cells param-slots + @ OP_STORE_DATA asm-emit-packed
  loop ;

: jl-parse-func ( -- )
  jl-next
  tok-type @ TK_IDENT <> if ." LOI: can ten ham" cr abort then
  save-tok-name jl-next
  jl-emit-jmp-ph func-jmp-patch ! jl-here func-entry-addr !
  jl-parse-params jl-emit-prologue
  saved-name saved-name-len @ func-entry-addr @ param-count @ func-add
  jl-skip-nl jl-parse-block
  0 OP_PUSH asm-emit-packed 0 OP_RET asm-emit-packed
  jl-here func-jmp-patch @ jl-patch
  tok-type @ TK_END <> if ." LOI: can 'end'" cr abort then jl-next ;

\ === Phân tích biểu thức (đệ quy giảm dần) ===

\ Thừa số: số | biến | hàm(args) | (expr) | -expr
: jl-parse-factor ( -- ) recursive
  tok-type @ TK_NUM = if tok-val @ OP_PUSH asm-emit-packed jl-next exit then
  tok-type @ TK_IDENT = if
    save-tok-name jl-next
    tok-type @ TK_LPAREN = if jl-emit-call exit then
    saved-name saved-name-len @ sym-find-or-add OP_LOAD_DATA asm-emit-packed exit then
  tok-type @ TK_LPAREN = if jl-next jl-parse-expr
    tok-type @ TK_RPAREN <> if ." LOI: can )" cr abort then jl-next exit then
  tok-type @ TK_MINUS = if jl-next 0 OP_PUSH asm-emit-packed
    jl-parse-factor 0 OP_SUB asm-emit-packed exit then
  ." LOI: bieu thuc khong hop le" cr abort ;

\ Tích: thừa_số { * thừa_số }
: jl-parse-term ( -- )
  jl-parse-factor
  begin tok-type @ TK_STAR = while
    jl-next jl-parse-factor 0 OP_MUL asm-emit-packed
  repeat ;

\ Tổng: tích { (+|-) tích }
: jl-parse-additive ( -- )
  jl-parse-term
  begin
    tok-type @ TK_PLUS = if jl-next jl-parse-term 0 OP_ADD asm-emit-packed true
    else tok-type @ TK_MINUS = if jl-next jl-parse-term 0 OP_SUB asm-emit-packed true
    else false then then
  while repeat ;

\ So sánh: tổng [ phép_so_sánh tổng ]
: jl-parse-comparison ( -- )
  jl-parse-additive
  tok-type @ TK_EQ  = if jl-next jl-parse-additive 0 OP_CMP_EQ asm-emit-packed exit then
  tok-type @ TK_NEQ = if jl-next jl-parse-additive
    0 OP_CMP_EQ asm-emit-packed 0 OP_PUSH asm-emit-packed 0 OP_CMP_EQ asm-emit-packed exit then
  tok-type @ TK_GT  = if jl-next jl-parse-additive 0 OP_CMP_GT asm-emit-packed exit then
  tok-type @ TK_LT  = if jl-next jl-parse-additive
    0 OP_SWAP asm-emit-packed 0 OP_CMP_GT asm-emit-packed exit then
  tok-type @ TK_GTE = if jl-next jl-parse-additive
    0 OP_SWAP asm-emit-packed 0 OP_CMP_GT asm-emit-packed
    0 OP_PUSH asm-emit-packed 0 OP_CMP_EQ asm-emit-packed exit then
  tok-type @ TK_LTE = if jl-next jl-parse-additive
    0 OP_CMP_GT asm-emit-packed 0 OP_PUSH asm-emit-packed 0 OP_CMP_EQ asm-emit-packed exit then ;

\ Gán thực thể cho biểu thức
:noname ( -- ) jl-parse-comparison ; is jl-parse-expr

\ === Phân tích câu lệnh ===
:noname ( -- )
  tok-type @ TK_NEWLINE  = if jl-next exit then
  tok-type @ TK_EOF      = if exit then
  tok-type @ TK_IF       = if jl-parse-if exit then
  tok-type @ TK_WHILE    = if jl-parse-while exit then
  tok-type @ TK_FUNCTION = if jl-parse-func exit then
  tok-type @ TK_RETURN   = if jl-parse-return exit then
  tok-type @ TK_PRINTLN  = if jl-parse-println exit then
  tok-type @ TK_IDENT    = if
    save-tok-name jl-next
    tok-type @ TK_ASSIGN = if
      saved-name saved-name-len @ sym-find-or-add >r
      jl-next jl-parse-expr r> OP_STORE_DATA asm-emit-packed exit then
    tok-type @ TK_LPAREN = if
      jl-emit-call 0 OP_DROP asm-emit-packed exit then
    ." LOI: cu phap sai sau ten bien" cr abort then
  ." LOI: cu phap khong hop le" cr abort
; is jl-parse-stmt

\ === Phân tích chương trình ===
: jl-parse-program ( -- )
  begin jl-skip-nl tok-type @ TK_EOF <> while jl-parse-stmt repeat ;

\ === Điểm vào chính ===
: jl-compile ( addr len -- )
  asm-begin sym-init func-init jl-len ! jl-src ! jl-next
  jl-parse-program 0 OP_HALT asm-emit-packed ;

: jl-run    ( addr len -- ) jl-compile 0 vpc ! vm-run ;
: jl-disasm ( addr len -- ) jl-compile find-prog-end prog-end @ disasm ;
: jl-debug  ( addr len -- ) jl-compile debugger-loop ;
