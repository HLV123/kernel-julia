\ ============================================================
\ 12-lexer.fs -- Bộ phân tích từ tố (Lexer)
\ Chuyển mã nguồn Julia thành luồng token
\ ============================================================

\ --- Loại token ---
0  constant TK_EOF       \ Hết mã nguồn
1  constant TK_NUM       \ Số nguyên
2  constant TK_IDENT     \ Tên biến / hàm
3  constant TK_ASSIGN    \ Phép gán =
4  constant TK_PLUS      \ Phép cộng +
5  constant TK_MINUS     \ Phép trừ -
6  constant TK_STAR      \ Phép nhân *
7  constant TK_LPAREN    \ Ngoặc mở (
8  constant TK_RPAREN    \ Ngoặc đóng )
9  constant TK_COMMA     \ Dấu phẩy ,
10 constant TK_NEWLINE   \ Xuống dòng hoặc ;
11 constant TK_EQ        \ So sánh ==
12 constant TK_NEQ       \ Khác !=
13 constant TK_GT        \ Lớn hơn >
14 constant TK_LT        \ Nhỏ hơn <
15 constant TK_GTE       \ Lớn bằng >=
16 constant TK_LTE       \ Nhỏ bằng <=
20 constant TK_IF        \ Từ khoá if
21 constant TK_ELSE      \ Từ khoá else
22 constant TK_ELSEIF    \ Từ khoá elseif
23 constant TK_END       \ Từ khoá end
24 constant TK_WHILE     \ Từ khoá while
25 constant TK_FUNCTION  \ Từ khoá function
26 constant TK_RETURN    \ Từ khoá return
27 constant TK_PRINTLN   \ Từ khoá println

\ --- Trạng thái nguồn ---
variable jl-src      \ Con trỏ đến mã nguồn hiện tại
variable jl-len      \ Số byte còn lại

\ --- Trạng thái token ---
variable tok-type    \ Loại token hiện tại
variable tok-val     \ Giá trị số (cho TK_NUM)
64 constant TOK-ID-MAX
create tok-id TOK-ID-MAX allot   \ Tên biến/hàm hiện tại
variable tok-id-len              \ Độ dài tên

\ --- Bộ đệm lưu tên (bảo vệ khi gọi hàm lồng) ---
64 constant SAVED-ID-MAX
create saved-name SAVED-ID-MAX allot
variable saved-name-len
: save-tok-name ( -- )
  tok-id-len @ saved-name-len !
  tok-id saved-name saved-name-len @ cmove ;

\ --- Hàm kiểm tra ký tự ---
: jl-eof? ( -- cờ ) jl-len @ 0<= ;
: jl-ch   ( -- c )  jl-src @ c@ ;
: jl-adv  ( -- )    jl-src @ 1+ jl-src ! jl-len @ 1- jl-len ! ;
: jl-is-ws? ( c -- cờ ) dup 32 = swap 9 = or ;
: jl-is-nl? ( c -- cờ ) dup 10 = swap 13 = or ;
: jl-is-digit? ( c -- cờ ) dup [char] 0 >= swap [char] 9 <= and ;
: jl-is-alpha? ( c -- cờ )
  dup [char] _ = if drop true exit then
  dup [char] a >= over [char] z <= and
  swap dup [char] A >= swap [char] Z <= and or ;
: jl-is-alnum? ( c -- cờ )
  dup jl-is-alpha? if drop true exit then jl-is-digit? ;

\ Bỏ qua khoảng trắng (không bỏ dòng mới)
: jl-skip-ws ( -- )
  begin jl-eof? 0= if jl-ch jl-is-ws? else false then while jl-adv repeat ;

\ Đọc số nguyên
: jl-read-num ( -- n )
  0 begin jl-eof? 0= if jl-ch jl-is-digit? else false then while
    10 * jl-ch [char] 0 - + jl-adv
  repeat ;

\ Đọc tên biến / hàm vào tok-id
: jl-read-id ( -- )
  0 tok-id-len !
  begin jl-eof? 0= if jl-ch jl-is-alnum? else false then while
    jl-ch tok-id tok-id-len @ + c! tok-id-len @ 1+ tok-id-len ! jl-adv
  repeat ;

\ Kiểm tra tên có phải từ khoá không
: jl-check-kw ( -- )
  tok-id tok-id-len @ s" if"       str= if TK_IF       tok-type ! exit then
  tok-id tok-id-len @ s" else"     str= if TK_ELSE     tok-type ! exit then
  tok-id tok-id-len @ s" elseif"   str= if TK_ELSEIF   tok-type ! exit then
  tok-id tok-id-len @ s" end"      str= if TK_END      tok-type ! exit then
  tok-id tok-id-len @ s" while"    str= if TK_WHILE    tok-type ! exit then
  tok-id tok-id-len @ s" function" str= if TK_FUNCTION tok-type ! exit then
  tok-id tok-id-len @ s" return"   str= if TK_RETURN   tok-type ! exit then
  tok-id tok-id-len @ s" println"  str= if TK_PRINTLN  tok-type ! exit then
  TK_IDENT tok-type ! ;

\ === Bộ phân tích chính – đọc token tiếp theo ===
: jl-next ( -- )
  jl-skip-ws
  jl-eof? if TK_EOF tok-type ! exit then
  jl-ch
  dup jl-is-nl? if drop jl-adv TK_NEWLINE tok-type ! exit then
  dup [char] ; = if drop jl-adv TK_NEWLINE tok-type ! exit then
  dup [char] # = if drop
    begin jl-eof? 0= if jl-ch jl-is-nl? 0= else false then while jl-adv repeat
    jl-eof? 0= if jl-adv then TK_NEWLINE tok-type ! exit then
  dup jl-is-digit? if drop jl-read-num tok-val ! TK_NUM tok-type ! exit then
  dup jl-is-alpha? if drop jl-read-id jl-check-kw exit then
  dup [char] + = if drop jl-adv TK_PLUS   tok-type ! exit then
  dup [char] - = if drop jl-adv TK_MINUS  tok-type ! exit then
  dup [char] * = if drop jl-adv TK_STAR   tok-type ! exit then
  dup [char] ( = if drop jl-adv TK_LPAREN tok-type ! exit then
  dup [char] ) = if drop jl-adv TK_RPAREN tok-type ! exit then
  dup [char] , = if drop jl-adv TK_COMMA  tok-type ! exit then
  dup [char] = = if drop jl-adv
    jl-eof? if TK_ASSIGN tok-type ! exit then
    jl-ch [char] = = if jl-adv TK_EQ tok-type ! else TK_ASSIGN tok-type ! then exit then
  dup [char] ! = if drop jl-adv
    jl-eof? 0= if jl-ch [char] = = if jl-adv TK_NEQ tok-type ! exit then then
    ." LOI: ky tu '!' khong hop le" cr abort then
  dup [char] > = if drop jl-adv
    jl-eof? if TK_GT tok-type ! exit then
    jl-ch [char] = = if jl-adv TK_GTE tok-type ! else TK_GT tok-type ! then exit then
  dup [char] < = if drop jl-adv
    jl-eof? if TK_LT tok-type ! exit then
    jl-ch [char] = = if jl-adv TK_LTE tok-type ! else TK_LT tok-type ! then exit then
  ." LOI: ky tu la '" emit ." '" cr abort ;
