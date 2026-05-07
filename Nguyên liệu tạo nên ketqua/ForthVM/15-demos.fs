\ ============================================================
\ 15-demos.fs -- Trình diễn tương tác cho 8 Phase
\ Menu cho người dùng trải nghiệm từng tính năng
\ ============================================================

\ --- Demo Phase 1: Phép tính 5 + 6 ---
: demo-1 ( -- )
  cr ." === Phase 1: Tinh 5 + 6 ===" cr
  ." Bytecode: PUSH 5, PUSH 6, ADD, PRINT, HALT" cr cr
  asm-begin s" PUSH 5 PUSH 6 ADD PRINT HALT" asm-eval
  ." Ket qua: " 0 vpc ! vm-run cr ;

\ --- Demo Phase 2: Đếm ngược 5 → 1 ---
: demo-2 ( -- )
  cr ." === Phase 2: Dem nguoc 5 -> 1 (vong lap) ===" cr
  ." Dung JZ va JMP de tao vong lap" cr cr
  asm-begin
  s" PUSH 5 POP_R 0 PUSH_R 0 DUP PRINT PUSH 1 SUB POP_R 0 PUSH_R 0 DUP JZ 13 JMP 2 DROP HALT"
  asm-eval 0 vpc ! vm-run cr ;

\ --- Demo Phase 3: Tính giai thừa 5! = 120 ---
: demo-3 ( -- )
  cr ." === Phase 3: Giai thua 5! = 120 (de quy) ===" cr
  ." Dung CALL/RET de goi ham de quy" cr cr
  asm-begin
  \ Dia chi: 0:PUSH 5  1:CALL 4  2:PRINT  3:HALT
  \          4:DUP  5:JZ 12  6:DUP  7:PUSH 1  8:SUB  9:CALL 4  10:MUL  11:RET
  \          12:DROP  13:PUSH 1  14:RET
  s" PUSH 5 CALL 4 PRINT HALT DUP JZ 12 DUP PUSH 1 SUB CALL 4 MUL RET DROP PUSH 1 RET"
  asm-eval
  ." Ket qua: " 0 vpc ! vm-run cr ;

\ --- Demo Phase 4: Debugger bước qua 5 + 6 ---
: demo-4 ( -- )
  cr ." === Phase 4: Debugger buoc tung lenh ===" cr
  ." An [n] de buoc tiep, [r] chay het, [q] thoat" cr cr
  asm-begin s" PUSH 5 PUSH 6 ADD PRINT HALT" asm-eval
  debugger-loop ;

\ --- Demo Phase 5: Bộ nhớ Data + Heap ---
: demo-5 ( -- )
  cr ." === Phase 5: Bo nho Data va Heap ===" cr
  ." Luu 42 vao Data slot 0, doc lai va in" cr cr
  asm-begin s" PUSH 42 STORE_DATA 0 LOAD_DATA 0 PRINT HALT" asm-eval
  ." Ket qua: " 0 vpc ! vm-run
  cr ." Cap phat 2 cells tren Heap:" cr
  heap-reset
  ." Truoc: heap-ptr = " heap-ptr @ . cr
  2 heap-alloc ." Sau:   heap-ptr = " heap-ptr @ . ."  (dia chi = " . ." )" cr cr ;

\ --- Demo Phase 6: Viết assembly trực tiếp ---
: demo-6 ( -- )
  cr ." === Phase 6: Hop dich van ban ===" cr
  ." Viet assembly: PUSH 10 PUSH 3 MUL PRINT HALT" cr cr
  asm-begin s" PUSH 10 PUSH 3 MUL PRINT HALT" asm-eval
  ." Bytecode da tao (" asm-count . ." lenh):" cr
  find-prog-end prog-end @ disasm
  cr ." Chay: " 0 vpc ! vm-run cr ;

\ --- Demo Phase 7: Xem trạng thái VM ---
: demo-7 ( -- )
  cr ." === Phase 7: Xem trang thai may ao ===" cr
  asm-begin s" PUSH 100 PUSH 200 PUSH 300 HALT" asm-eval
  0 vpc ! vm-run
  .vm-state cr ;

\ --- Demo Phase 8: Viết code Julia ---
: demo-8 ( -- )
  cr ." === Phase 8: Ngon ngu Julia Tiny ===" cr
  ." Cac vi du:" cr
  cr ." 1) Phep tinh:" cr
  s" println((2 + 3) * 4)" jl-run
  cr ." 2) Bien va vong lap:" cr
  s" i = 5 ; while i > 0 ; println(i) ; i = i - 1 ; end" jl-run
  cr ." 3) Ham de quy tinh giai thua:" cr
  s" function fact(n) ; if n == 1 ; return 1 ; end ; return n * fact(n - 1) ; end ; println(fact(6))" jl-run
  cr ." Ban co the tu viet code Julia trong REPL!" cr
  ."   Go: s" [char] " emit ."  <code> " [char] " emit ."  jl-run" cr cr ;

\ --- Julia REPL ---
1024 constant JL-LINE-MAX
create jl-line JL-LINE-MAX allot
variable jl-line-len

variable repl-quit

: repl-is-quit? ( -- flag )
  jl-line-len @ 4 =
  jl-line     c@ [char] q = and
  jl-line 1 + c@ [char] u = and
  jl-line 2 + c@ [char] i = and
  jl-line 3 + c@ [char] t = and ;

: repl-do-run ( -- )
  jl-line jl-line-len @ jl-run ;

: repl-run-line ( -- )
  jl-line-len @ 0> if repl-do-run then ;

: repl-step ( -- )
  ." julia> "
  jl-line JL-LINE-MAX accept jl-line-len !
  repl-is-quit? if 1 repl-quit ! else repl-run-line then ;

: julia-repl ( -- )
  cr ." === Julia REPL (go 'quit' de thoat) ===" cr
  0 repl-quit !
  begin repl-quit @ 0= while
    repl-step
  repeat
  cr ." Thoat REPL." cr ;

\ === Menu chính ===
: show-menu ( -- )
  cr
  ." ======================================================" cr
  ."   FORTHVM -- May Ao Tu Xay Dung Bang Forth" cr
  ."   Phien ban hoan chinh: Phase 1-8" cr
  ." ======================================================" cr
  ."   [1] Phase 1: Tinh 5 + 6 (ngan xep + phep tinh)" cr
  ."   [2] Phase 2: Dem nguoc 5->1 (vong lap)" cr
  ."   [3] Phase 3: Giai thua 5! (goi ham de quy)" cr
  ."   [4] Phase 4: Debugger buoc tung lenh" cr
  ."   [5] Phase 5: Bo nho Data + Heap" cr
  ."   [6] Phase 6: Hop dich van ban assembly" cr
  ."   [7] Phase 7: Xem trang thai may ao" cr
  ."   [8] Phase 8: Ngon ngu Julia Tiny" cr
  ."   [j] Julia REPL (tu viet code truc tiep)" cr
  ."   [q] Thoat" cr
  ." ======================================================" cr
  ." > " ;

: run-menu ( -- )
  begin
    show-menu read-cmd
    dup [char] 1 = if drop demo-1 else
    dup [char] 2 = if drop demo-2 else
    dup [char] 3 = if drop demo-3 else
    dup [char] 4 = if drop demo-4 else
    dup [char] 5 = if drop demo-5 else
    dup [char] 6 = if drop demo-6 else
    dup [char] 7 = if drop demo-7 else
    dup [char] 8 = if drop demo-8 else
    dup [char] j = if drop julia-repl else
    dup [char] q = if drop cr ." Tam biet!" cr bye else
    drop ." Lua chon khong hop le, thu lai." cr
    then then then then then then then then then then
  again ;
