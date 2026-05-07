// ============================================================
// demos.rs — Cài đặt demo files vào VFS
// ============================================================

use alloc::string::String;

/// Cài đặt tất cả demo .jl files vào /etc/julia/
pub fn install_demo_files() {
    let _ = crate::fs::mkdir("/etc");
    let _ = crate::fs::mkdir("/etc/julia");

    let _ = crate::fs::write_file("/etc/julia/hello.jl",
b"# Hello World
println(\"Hello from Julia Tiny on MyKernel!\")
println(\"Version: 0.2\")
println(\"Type 'help' in REPL for features\")
");

    let _ = crate::fs::write_file("/etc/julia/arithmetic.jl",
b"# Arithmetic demo
println(\"=== Arithmetic ===\")
println(\"2 + 3 = \", 2 + 3)
println(\"10 - 4 = \", 10 - 4)
println(\"6 * 7 = \", 6 * 7)
println(\"17 / 5 = \", 17 / 5)
println(\"17 % 5 = \", 17 % 5)
println(\"2 ^ 10 = \", 2 ^ 10)
println(\"-42 = \", -42)
println(\"abs(-42) = \", abs(-42))
");

    let _ = crate::fs::write_file("/etc/julia/fibonacci.jl",
b"# Fibonacci sequence
function fib(n)
  if n <= 1
    return n
  end
  return fib(n - 1) + fib(n - 2)
end

println(\"Fibonacci sequence:\")
for i = 0:12
  print(\"fib(\", i, \") = \")
  println(fib(i))
end
");

    let _ = crate::fs::write_file("/etc/julia/factorial.jl",
b"# Factorial
function fact(n)
  if n <= 1
    return 1
  end
  return n * fact(n - 1)
end

for i = 1:10
  println(\"fact(\", i, \") = \", fact(i))
end
");

    let _ = crate::fs::write_file("/etc/julia/fizzbuzz.jl",
b"# FizzBuzz
for i = 1:30
  if i % 15 == 0
    println(\"FizzBuzz\")
  elseif i % 3 == 0
    println(\"Fizz\")
  elseif i % 5 == 0
    println(\"Buzz\")
  else
    println(i)
  end
end
");

    let _ = crate::fs::write_file("/etc/julia/strings.jl",
b"# String demo
name = \"MyKernel\"
version = \"0.2\"
println(\"Name: $name\")
println(\"Version: $version\")
println(\"Length: \", length(name))

greeting = \"Hello\" * \" \" * \"World!\"
println(greeting)

println(uppercase(name))
println(repeat(\"=-\", 15))
");

    let _ = crate::fs::write_file("/etc/julia/arrays.jl",
b"# Array demo
a = [10, 20, 30, 40, 50]
println(\"Array: \", a)
println(\"Length: \", length(a))
println(\"a[1] = \", a[1])
println(\"a[3] = \", a[3])

push!(a, 60)
println(\"After push: \", a)

println(\"Sum: \", sum(a))
println(\"Max: \", maximum(a))
println(\"Min: \", minimum(a))

sort!(a)
println(\"Sorted: \", a)
");

    let _ = crate::fs::write_file("/etc/julia/forloop.jl",
b"# For loop demos
println(\"Count 1 to 5:\")
for i = 1:5
  println(i)
end

println(\"Even numbers 0 to 20:\")
for i = 0:2:20
  print(i, \" \")
end
println(\"\")

println(\"Countdown:\")
for i = 10:-1:1
  print(i, \" \")
end
println(\"Go!\")
");

    let _ = crate::fs::write_file("/etc/julia/fileio.jl",
b"# File I/O demo
write_file(\"/tmp/hello.txt\", \"Hello from Julia!\\n\")
println(\"Wrote /tmp/hello.txt\")

content = read_file(\"/tmp/hello.txt\")
println(\"Read back: \", content)

if file_exists(\"/tmp/hello.txt\")
  println(\"File exists!\")
end
");

    let _ = crate::fs::write_file("/etc/julia/system.jl",
b"# System info
println(\"Uptime: \", uptime(), \" seconds\")
println(\"Ticks: \", ticks())
println(\"Random: \", random())
println(\"Random: \", random())
println(\"Random: \", random())
");

    let _ = crate::fs::write_file("/etc/julia/primes.jl",
b"# Find primes up to N
function is_prime(n)
  if n < 2
    return false
  end
  i = 2
  while i * i <= n
    if n % i == 0
      return false
    end
    i += 1
  end
  return true
end

println(\"Primes up to 50:\")
count = 0
for n = 2:50
  if is_prime(n)
    print(n, \" \")
    count += 1
  end
end
println(\"\")
println(\"Found \", count, \" primes\")
");
}
