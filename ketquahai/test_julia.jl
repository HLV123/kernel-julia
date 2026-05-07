# ============================================================
# test_julia.jl -- Full feature test for Julia Tiny v0.2
# Chạy: include("test_julia.jl") hoặc paste từng block
# ============================================================

# ------------------------------------------------------------
# 1. ARITHMETIC & BASIC TYPES
# ------------------------------------------------------------
println("=== 1. Arithmetic ===")
a = 10
b = 3
println(a + b)       # 13
println(a - b)       # 7
println(a * b)       # 30
println(a / b)       # 3.333...
println(a % b)       # 1
println(a ^ b)       # 1000

# ------------------------------------------------------------
# 2. ASSIGNMENT OPERATORS
# ------------------------------------------------------------
println("=== 2. Assignment ops ===")
x = 5
x += 3
println(x)   # 8
x -= 2
println(x)   # 6
x *= 4
println(x)   # 24
x /= 6
println(x)   # 4
x %= 3
println(x)   # 1

# ------------------------------------------------------------
# 3. COMPARISON & LOGIC
# ------------------------------------------------------------
println("=== 3. Compare & Logic ===")
println(2 == 2)    # true
println(2 != 3)    # true
println(5 > 3)     # true
println(5 < 3)     # false
println(4 >= 4)    # true
println(3 <= 2)    # false
println(true && false)  # false
println(true || false)  # true
println(!true)          # false

# ------------------------------------------------------------
# 4. BITWISE
# ------------------------------------------------------------
println("=== 4. Bitwise ===")
println(6 & 3)    # 2
println(6 | 3)    # 7
println(1 << 3)   # 8
println(16 >> 2)  # 4

# ------------------------------------------------------------
# 5. STRINGS & INTERPOLATION
# ------------------------------------------------------------
println("=== 5. Strings ===")
name = "MyKernel"
println("Hello $name")
println(length("hello"))          # 5
println(uppercase("tiny julia"))  # TINY JULIA

# ------------------------------------------------------------
# 6. ARRAYS
# ------------------------------------------------------------
println("=== 6. Arrays ===")
arr = [10, 20, 30]
println(arr[1])        # 10
println(length(arr))   # 3
push!(arr, 40)
println(length(arr))   # 4
println(arr[4])        # 40

# ------------------------------------------------------------
# 7. IF / ELSEIF / ELSE / END
# ------------------------------------------------------------
println("=== 7. If/elseif/else ===")
score = 75
if score >= 90
    println("A")
elseif score >= 75
    println("B")
elseif score >= 60
    println("C")
else
    println("F")
end

# ------------------------------------------------------------
# 8. WHILE + BREAK + CONTINUE
# ------------------------------------------------------------
println("=== 8. While / break / continue ===")
i = 0
while i < 10
    i += 1
    if i == 3
        continue
    end
    if i == 6
        break
    end
    print(i)
    print(" ")
end
println("")   # newline; output: 1 2 4 5

# ------------------------------------------------------------
# 9. FOR + RANGE + BREAK + CONTINUE
# ------------------------------------------------------------
println("=== 9. For loop ===")
for i = 1:8
    if i % 2 == 0
        continue
    end
    if i > 7
        break
    end
    print(i)
    print(" ")
end
println("")   # 1 3 5 7

# ------------------------------------------------------------
# 10. FUNCTIONS + RETURN
# ------------------------------------------------------------
println("=== 10. Functions ===")
function add(a, b)
    return a + b
end

function factorial(n)
    if n <= 1
        return 1
    end
    return n * factorial(n - 1)
end

function fib(n)
    if n <= 1
        return n
    end
    a = 0
    b = 1
    k = 2
    while k <= n
        c = a + b
        a = b
        b = c
        k += 1
    end
    return b
end

println(add(4, 7))        # 11
println(factorial(6))     # 720
println(fib(10))          # 55

# ------------------------------------------------------------
# 11. MATH BUILTINS
# ------------------------------------------------------------
println("=== 11. Math ===")
println(abs(-42))          # 42
println(max(3, 7))         # 7
println(min(3, 7))         # 3
println(sqrt(144))         # 12
println(gcd(48, 18))       # 6
println(clamp(15, 0, 10))  # 10
println(clamp(-5, 0, 10))  # 0

# ------------------------------------------------------------
# 12. SYSTEM BUILTINS
# ------------------------------------------------------------
println("=== 12. System ===")
t1 = ticks()
println(t1 >= 0)    # true
println(uptime() >= 0)  # true
r = random()
println(r >= 0)     # true
sleep(0)            # should not crash

# ------------------------------------------------------------
# 13. NESTED FUNCTIONS & CLOSURES
# ------------------------------------------------------------
println("=== 13. Nested logic ===")
function is_prime(n)
    if n < 2
        return false
    end
    d = 2
    while d * d <= n
        if n % d == 0
            return false
        end
        d += 1
    end
    return true
end

for i = 1:20
    if is_prime(i)
        print(i)
        print(" ")
    end
end
println("")   # 2 3 5 7 11 13 17 19

# ------------------------------------------------------------
# 14. ARRAY ALGORITHMS
# ------------------------------------------------------------
println("=== 14. Array algorithms ===")
function bubble_sort(arr)
    n = length(arr)
    i = 1
    while i <= n
        j = 1
        while j <= n - i
            if arr[j] > arr[j+1]
                tmp = arr[j]
                arr[j] = arr[j+1]
                arr[j+1] = tmp
            end
            j += 1
        end
        i += 1
    end
    return arr
end

data = [5, 2, 8, 1, 9, 3]
sorted = bubble_sort(data)
for i = 1:length(sorted)
    print(sorted[i])
    print(" ")
end
println("")   # 1 2 3 5 8 9

# ------------------------------------------------------------
# 15. STRING OPERATIONS
# ------------------------------------------------------------
println("=== 15. String ops ===")
s = "hello world"
println(length(s))           # 11
println(uppercase(s))        # HELLO WORLD
greeting = "world"
println("hi $greeting!")     # hi world!

# ------------------------------------------------------------
# 16. FILE I/O
# ------------------------------------------------------------
println("=== 16. File I/O ===")
write_file("/tmp/mytest.txt", "kernel rocks")
println(file_exists("/tmp/mytest.txt"))
content = read_file("/tmp/mytest.txt")
println(content)
println(file_exists("/tmp/no_such.txt"))

# ------------------------------------------------------------
# 17. VARS & FUNCS COMMANDS (gõ thủ công trong REPL)
# ------------------------------------------------------------
# Sau khi chạy file này, gõ:
#   vars   -- xem biến: a b arr data greeting i name ...
#   funcs  -- xem hàm: add factorial fib is_prime bubble_sort

# ------------------------------------------------------------
# 18. STRESS: NESTED LOOPS & ACCUMULATOR
# ------------------------------------------------------------
println("=== 18. Nested loops ===")
sum = 0
for i = 1:10
    for j = 1:10
        sum += i * j
    end
end
println(sum)   # 3025

# ------------------------------------------------------------
# DONE
# ------------------------------------------------------------
println("=== ALL TESTS PASSED ===")
