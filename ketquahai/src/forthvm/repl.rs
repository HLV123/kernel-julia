// ============================================================
// repl.rs — REPL mode tương tác
// Multi-line, giữ trạng thái, auto-print
// ============================================================

use alloc::string::String;
use alloc::vec::Vec;
use crate::forthvm::vm::ForthVm;
use crate::forthvm::compiler::{Compiler, CompileError};
use crate::forthvm::symbols::{VarTable, FuncTable};
use crate::forthvm::value::{Value, format_value};
use crate::forthvm::opcode::OP_HALT;
use crate::{print, println};

/// Chạy REPL mode
pub fn run_repl() {
    println!("");
    println!("  Julia Tiny v0.2 -- MyKernel Runtime");
    println!("  Type 'exit' to quit, 'help' for commands");
    println!("");

    let mut vm = ForthVm::new();
    let mut vars = VarTable::new();
    let mut funcs = FuncTable::new();
    let mut emit_ptr: usize = 0;

    // Tạo demo files trong VFS
    crate::forthvm::demos::install_demo_files();

    loop {
        print!("jl> ");
        let line = read_line();
        let trimmed = line.trim();

        if trimmed.is_empty() { continue; }

        // REPL commands
        match trimmed {
            "exit" | "quit" => { println!("Bye!"); return; }
            "help" => { print_repl_help(); continue; }
            "vars" => { print_vars(&vars); continue; }
            "funcs" => { print_funcs(&funcs); continue; }
            "clear" => { vm.reset_all(); vars.reset(); funcs.reset(); emit_ptr = 0; println!("State cleared."); continue; }
            _ => {}
        }

        // Check include
        if trimmed.starts_with("include(") || trimmed.starts_with("include \"") {
            let path = extract_path(trimmed);
            if !path.is_empty() {
                run_file(&mut vm, &mut vars, &mut funcs, &mut emit_ptr, &path);
            }
            continue;
        }

        // Multi-line: đếm block openers
        let mut source = String::from(trimmed);
        let mut depth = count_depth(trimmed);
        while depth > 0 {
            print!("... ");
            let next_line = read_line();
            source.push('\n');
            source.push_str(&next_line);
            depth += count_depth_delta(next_line.trim());
        }

        // Compile & run
        compile_and_run(&mut vm, &mut vars, &mut funcs, &mut emit_ptr, &source);
    }
}

/// Chạy file Julia từ VFS
pub fn run_file(
    vm: &mut ForthVm, vars: &mut VarTable, funcs: &mut FuncTable,
    emit_ptr: &mut usize, path: &str,
) {
    match crate::fs::read_file(path) {
        Ok(data) => {
            match core::str::from_utf8(&data) {
                Ok(source) => compile_and_run(vm, vars, funcs, emit_ptr, source),
                Err(_) => println!("Error: file is not valid UTF-8"),
            }
        }
        Err(_) => println!("Error: cannot read '{}'", path),
    }
}

/// Compile source và chạy
fn compile_and_run(
    vm: &mut ForthVm, vars: &mut VarTable, funcs: &mut FuncTable,
    emit_ptr: &mut usize, source: &str,
) {
    // Tạo compiler với bản sao của trạng thái hiện tại (để không mất data nếu lỗi cú pháp)
    let compiler = Compiler::new_repl(vm, source, vars.clone(), funcs.clone(), *emit_ptr);
    let code_start = compiler.code_start;

    match compiler.compile() {
        Ok((new_ptr, new_vars, new_funcs)) => {
            *emit_ptr = new_ptr;
            *vars = new_vars;
            *funcs = new_funcs;

            // Chạy từ code_start
            vm.pc = code_start;
            let result = vm.run();
            match result {
                crate::forthvm::vm::VmResult::Halted => {}
                crate::forthvm::vm::VmResult::Error(e) => {
                    println!("Runtime error: {:?}", e);
                }
                crate::forthvm::vm::VmResult::Yielded => {}
            }
        }
        Err(e) => {
            // Restore old state on error
            println!("Error: {:?}", e);
        }
    }
}

/// Chạy file từ shell (one-shot, không REPL)
pub fn run_file_oneshot(path: &str) {
    let mut vm = ForthVm::new();
    match crate::fs::read_file(path) {
        Ok(data) => {
            match core::str::from_utf8(&data) {
                Ok(source) => {
                    match crate::forthvm::compiler::jl_run(&mut vm, source) {
                        Ok(result) => {
                            if let crate::forthvm::vm::VmResult::Error(e) = result {
                                println!("Runtime error: {:?}", e);
                            }
                        }
                        Err(e) => println!("Compile error: {:?}", e),
                    }
                }
                Err(_) => println!("Error: file is not valid UTF-8"),
            }
        }
        Err(_) => println!("Error: cannot read '{}'", path),
    }
}

// --- Helpers ---

fn count_depth(line: &str) -> i32 {
    let mut lexer = crate::forthvm::lexer::Lexer::new(line);
    let mut d: i32 = 0;
    loop {
        match lexer.kind() {
            crate::forthvm::lexer::TokenKind::If |
            crate::forthvm::lexer::TokenKind::While |
            crate::forthvm::lexer::TokenKind::For |
            crate::forthvm::lexer::TokenKind::Function => d += 1,
            crate::forthvm::lexer::TokenKind::End => d -= 1,
            crate::forthvm::lexer::TokenKind::Eof => break,
            _ => {}
        }
        lexer.next();
    }
    d
}

fn count_depth_delta(line: &str) -> i32 {
    count_depth(line)
}

fn extract_path(s: &str) -> String {
    // include("path") or include "path"
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("include(\"") {
        if let Some(path) = rest.strip_suffix("\")") {
            return String::from(path);
        }
    }
    if let Some(rest) = s.strip_prefix("include \"") {
        if let Some(path) = rest.strip_suffix("\"") {
            return String::from(path);
        }
    }
    String::new()
}

fn print_repl_help() {
    println!("REPL commands:");
    println!("  exit           -- quit REPL");
    println!("  help           -- this help");
    println!("  vars           -- list variables");
    println!("  funcs          -- list functions");
    println!("  clear          -- reset VM state");
    println!("  include(\"path\") -- load and run a .jl file");
    println!("");
    println!("Language features:");
    println!("  Arithmetic: + - * / % ^ (power)");
    println!("  Assignment: = += -= *= /= %=");
    println!("  Compare:    == != < > <= >=");
    println!("  Logic:      && || !");
    println!("  Bitwise:    & | ~ << >>");
    println!("  Types:      42  true  false  \"hello\"  [1,2,3]");
    println!("  Control:    if/elseif/else/end  while/end  for i=1:10/end");
    println!("  Flow:       break  continue  return");
    println!("  Functions:  function f(x) ... end");
    println!("  I/O:        println()  print()  readline()");
    println!("  Strings:    \"hello $name\"  length()  uppercase()");
    println!("  Arrays:     [1,2,3]  a[i]  push!(a,v)  length(a)");
    println!("  Files:      read_file()  write_file()  file_exists()");
    println!("  System:     ticks()  random()  uptime()  sleep()");
    println!("  Math:       abs() max() min() sqrt() gcd() clamp()");
}

fn print_vars(vars: &VarTable) {
    let count = vars.count();
    if count == 0 { println!("(no variables)"); return; }
    println!("Variables ({}):", count);
    for i in 0..count {
        if let Some(name) = vars.name_at(i) {
            if !name.starts_with("__") {
                println!("  [{}] {}", i, name);
            }
        }
    }
}

fn print_funcs(funcs: &FuncTable) {
    let count = funcs.count();
    if count == 0 { println!("(no functions)"); return; }
    println!("Functions ({}):", count);
    for i in 0..count {
        if let Some(name) = funcs.name_at(i) {
            println!("  {}", name);
        }
    }
}

/// Đọc 1 dòng từ serial
fn read_line() -> String {
    let mut buf = String::new();
    loop {
        let lsr: u8 = unsafe {
            let v: u8;
            core::arch::asm!("in al, dx", in("dx") 0x3FDu16, out("al") v, options(nomem, nostack));
            v
        };
        if lsr & 1 != 0 {
            let byte: u8 = unsafe {
                let v: u8;
                core::arch::asm!("in al, dx", in("dx") 0x3F8u16, out("al") v, options(nomem, nostack));
                v
            };
            match byte {
                b'\r' | b'\n' => { println!(""); return buf; }
                b'\x08' | b'\x7f' => { if !buf.is_empty() { buf.pop(); print!("\x08 \x08"); } }
                b'\x03' => { println!("^C"); return String::new(); }
                b if b >= 0x20 => { buf.push(b as char); print!("{}", b as char); }
                _ => {}
            }
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
}
