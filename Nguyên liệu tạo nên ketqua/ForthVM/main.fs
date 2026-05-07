\ ============================================================
\ main.fs -- Điểm vào chính
\ Nạp tất cả module theo đúng thứ tự phụ thuộc, hiện menu
\ Chạy: gforth main.fs
\ ============================================================

\ --- Phase 1: Lõi máy ảo ---
include 00-constants.fs    \ Hằng số hệ thống
include 01-memory.fs       \ Bố trí bộ nhớ
include 02-stack.fs        \ Ngăn xếp dữ liệu
include 03-registers.fs    \ Thanh ghi R0–R7
include 04-program.fs      \ Bộ nhớ chương trình + fetch

\ --- Phase 3: Gọi hàm ---
include 05-callstack.fs    \ Ngăn xếp lời gọi

\ --- Phase 5: Phân vùng ---
include 06-segments.fs     \ Data + Heap

\ --- Phase 1-8: Xử lý lệnh ---
include 07-handlers.fs     \ Bộ xử lý lệnh
include 08-dispatch.fs     \ Phân phối + vòng lặp

\ --- Phase 4: Gỡ lỗi ---
include 09-disasm.fs       \ Trình dịch ngược
include 10-debugger.fs     \ Trình gỡ lỗi

\ --- Phase 6: Hợp dịch ---
include 11-assembler.fs    \ Trình hợp dịch văn bản

\ --- Phase 8: Ngôn ngữ Julia ---
include 12-lexer.fs        \ Bộ phân tích từ tố
include 13-symbols.fs      \ Bảng ký hiệu
include 14-compiler.fs     \ Trình biên dịch

\ --- Giao diện ---
include 15-demos.fs        \ Menu tương tác

\ === Khởi chạy ===
run-menu
