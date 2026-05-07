\ run.fs -- Chay nhieu demo Julia
\ Cach dung: gforth run.fs

include 00-constants.fs
include 01-memory.fs
include 02-stack.fs
include 03-registers.fs
include 04-program.fs
include 05-callstack.fs
include 06-segments.fs
include 07-handlers.fs
include 08-dispatch.fs
include 09-disasm.fs
include 10-debugger.fs
include 11-assembler.fs
include 12-lexer.fs
include 13-symbols.fs
include 14-compiler.fs

\ --- 1. Phep tinh co ban ---
s" println((10 + 5) * 3)" jl-run

\ --- 2. Tong 1+2+...+100 ---
s" i = 1 ; s = 0 ; while i <= 100 ; s = s + i ; i = i + 1 ; end ; println(s)" jl-run

\ --- 3. Luy thua 2^10 ---
s" b = 2 ; r = 1 ; i = 0 ; while i < 10 ; r = r * b ; i = i + 1 ; end ; println(r)" jl-run

\ --- 4. Giai thua 10! ---
s" function fact(n) ; if n == 1 ; return 1 ; end ; return n * fact(n-1) ; end ; println(fact(10))" jl-run

\ --- 5. Fibonacci lan luot (lap, khong de quy doi) ---
s" function fib(n) ; a = 0 ; b = 1 ; i = 0 ; while i < n ; t = a + b ; a = b ; b = t ; i = i + 1 ; end ; return a ; end ; println(fib(10))" jl-run

\ --- 6. Tong binh phuong 1^2+...+10^2 ---
s" function sq(n) ; return n * n ; end ; i = 1 ; s = 0 ; while i <= 10 ; s = s + sq(i) ; i = i + 1 ; end ; println(s)" jl-run

\ --- 7. So lon nhat (max) ---
s" function max2(a,b) ; if a > b ; return a ; end ; return b ; end ; println(max2(max2(12,7),9))" jl-run

\ --- 8. So nho nhat (min) ---
s" function min2(a,b) ; if a < b ; return a ; end ; return b ; end ; println(min2(min2(12,7),9))" jl-run

\ --- 9. Dem so le tu 1 den 20 ---
s" i = 1 ; c = 0 ; while i <= 20 ; c = c + 1 ; i = i + 2 ; end ; println(c)" jl-run

\ --- 10. Nhan chuoi so: 1*2*3*...*8 ---
s" r = 1 ; i = 1 ; while i <= 8 ; r = r * i ; i = i + 1 ; end ; println(r)" jl-run

\ --- 11. Gia tri tuyet doi (abs) ---
s" function myabs(n) ; if n < 0 ; return 0 - n ; end ; return n ; end ; println(myabs(0-42))" jl-run

\ --- 12. Kiem tra so nguyen to (13) ---
s" function isprime(n) ; d = 2 ; r = 1 ; while d * d <= n ; if n - d * (n - d * 1) == 0 ; r = 0 ; end ; d = d + 1 ; end ; return r ; end ; println(isprime(13))" jl-run

\ --- 13. Tong cap so nhan: 1+3+9+27+81 (cong bi=3, 5 so hang) ---
s" s = 0 ; t = 1 ; i = 0 ; while i < 5 ; s = s + t ; t = t * 3 ; i = i + 1 ; end ; println(s)" jl-run

\ --- 14. Dem chu so trong 12345 (dem so lan nhan voi 1) ---
s" n = 12345 ; c = 0 ; while n > 0 ; c = c + 1 ; n = n - 1 ; end ; println(c)" jl-run

\ --- 15. Ham tinh max cua 3 so ---
s" function max2(a,b) ; if a > b ; return a ; end ; return b ; end ; function max3(a,b,c) ; return max2(max2(a,b),c) ; end ; println(max3(5,11,8))" jl-run

bye
