\ ============================================================
\ 13-symbols.fs -- Bảng ký hiệu
\ Quản lý tên biến → slot dữ liệu, tên hàm → địa chỉ code
\ ============================================================

\ === Bảng biến (tối đa 64 biến) ===
64 constant MAX-VARS
32 constant VAR-NAME-MAX
create sym-names MAX-VARS VAR-NAME-MAX * allot  \ Tên các biến
create sym-nlens MAX-VARS cells allot           \ Độ dài tên
variable sym-count                              \ Số biến hiện có

: sym-init ( -- ) 0 sym-count ! ;

\ Tìm biến theo tên → trả slot nếu thấy
: sym-find ( addr len -- slot đúng | sai )
  sym-count @ 0 ?do
    2dup i VAR-NAME-MAX * sym-names + i cells sym-nlens + @ str=
    if 2drop i true unloop exit then
  loop 2drop false ;

\ Thêm biến mới → trả slot vừa tạo
: sym-add ( addr len -- slot )
  sym-count @ MAX-VARS >= if ." LOI: qua nhieu bien" cr abort then
  dup sym-count @ cells sym-nlens + !
  sym-count @ VAR-NAME-MAX * sym-names + swap cmove
  sym-count @ sym-count @ 1+ sym-count ! ;

\ Tìm hoặc tạo mới nếu chưa có
: sym-find-or-add ( addr len -- slot )
  2dup sym-find if nip nip exit then sym-add ;

\ === Bảng hàm (tối đa 16 hàm) ===
16 constant MAX-FUNCS
32 constant FUNC-NAME-MAX
create func-names MAX-FUNCS FUNC-NAME-MAX * allot  \ Tên hàm
create func-nlens MAX-FUNCS cells allot            \ Độ dài tên
create func-addrs MAX-FUNCS cells allot            \ Địa chỉ bytecode
create func-pcnts MAX-FUNCS cells allot            \ Số tham số
variable func-count

: func-init ( -- ) 0 func-count ! ;

\ Đăng ký hàm mới
: func-add ( addr len code-addr param-count -- )
  func-count @ MAX-FUNCS >= if ." LOI: qua nhieu ham" cr abort then
  func-count @ cells func-pcnts + !
  func-count @ cells func-addrs + !
  dup func-count @ cells func-nlens + !
  func-count @ FUNC-NAME-MAX * func-names + swap cmove
  func-count @ 1+ func-count ! ;

\ Tìm hàm → trả địa chỉ code và số tham số
: func-find ( addr len -- code-addr param-count đúng | sai )
  func-count @ 0 ?do
    2dup i FUNC-NAME-MAX * func-names + i cells func-nlens + @ str=
    if 2drop i cells func-addrs + @ i cells func-pcnts + @ true unloop exit then
  loop 2drop false ;
