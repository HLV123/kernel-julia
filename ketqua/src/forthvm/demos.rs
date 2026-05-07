// ============================================================
// demos.rs -- Trình diễn tương tác cho 8 Phase
// Các chương trình mẫu cho từng tính năng
// (chuyển từ 15-demos.fs sang Rust)
// ============================================================

use crate::forthvm::vm::ForthVm;
use crate::forthvm::assembler;
use crate::forthvm::compiler;

/// Demo Phase 1: Tính 5 + 6
pub fn demo_phase1(vm: &mut ForthVm) {
    crate::println!("=== Phase 1: Tinh 5 + 6 ===");
    crate::println!("Bytecode: PUSH 5, PUSH 6, ADD, PRINT, HALT");
    let _ = assembler::assemble_into(vm, "PUSH 5 PUSH 6 ADD PRINT HALT");
    crate::print!("Ket qua: ");
    vm.run();
}

/// Demo Phase 2: Đếm ngược 5 → 1
pub fn demo_phase2(vm: &mut ForthVm) {
    crate::println!("=== Phase 2: Dem nguoc 5 -> 1 (vong lap) ===");
    crate::println!("Dung JZ va JMP de tao vong lap");
    // PUSH 5, POP_R 0, PUSH_R 0, DUP, PRINT, PUSH 1, SUB, POP_R 0,
    // PUSH_R 0, DUP, JZ 13, JMP 2, DROP, HALT
    let _ = assembler::assemble_into(vm,
        "PUSH 5 POP_R 0 PUSH_R 0 DUP PRINT PUSH 1 SUB POP_R 0 PUSH_R 0 DUP JZ 13 JMP 2 DROP HALT"
    );
    vm.run();
}

/// Demo Phase 3: Tính giai thừa 5! = 120
pub fn demo_phase3(vm: &mut ForthVm) {
    crate::println!("=== Phase 3: Giai thua 5! = 120 (de quy) ===");
    crate::println!("Dung CALL/RET de goi ham de quy");
    // Addr: 0:PUSH 5  1:CALL 4  2:PRINT  3:HALT
    //       4:DUP  5:JZ 12  6:DUP  7:PUSH 1  8:SUB  9:CALL 4  10:MUL  11:RET
    //       12:DROP  13:PUSH 1  14:RET
    let _ = assembler::assemble_into(vm,
        "PUSH 5 CALL 4 PRINT HALT DUP JZ 12 DUP PUSH 1 SUB CALL 4 MUL RET DROP PUSH 1 RET"
    );
    crate::print!("Ket qua: ");
    vm.run();
}

/// Demo Phase 5: Bộ nhớ Data + Heap
pub fn demo_phase5(vm: &mut ForthVm) {
    crate::println!("=== Phase 5: Bo nho Data va Heap ===");
    crate::println!("Luu 42 vao Data slot 0, doc lai va in");
    let _ = assembler::assemble_into(vm,
        "PUSH 42 STORE_DATA 0 LOAD_DATA 0 PRINT HALT"
    );
    crate::print!("Ket qua: ");
    vm.run();
}

/// Demo Phase 6: Viết assembly trực tiếp
pub fn demo_phase6(vm: &mut ForthVm) {
    crate::println!("=== Phase 6: Hop dich van ban ===");
    crate::println!("Viet assembly: PUSH 10 PUSH 3 MUL PRINT HALT");
    let count = assembler::assemble_into(vm,
        "PUSH 10 PUSH 3 MUL PRINT HALT"
    ).unwrap_or(0);
    crate::println!("Bytecode da tao ({} lenh):", count);
    vm.find_prog_end();
    crate::forthvm::disasm::disasm_print(vm, vm.prog_end);
    crate::print!("Chay: ");
    vm.pc = 0;
    vm.run();
}

/// Demo Phase 8: Viết code Julia
pub fn demo_phase8(vm: &mut ForthVm) {
    crate::println!("=== Phase 8: Ngon ngu Julia Tiny ===");

    crate::println!("1) Phep tinh: println((2 + 3) * 4)");
    let _ = compiler::jl_run(vm, "println((2 + 3) * 4)");

    crate::println!("2) Bien va vong lap:");
    let _ = compiler::jl_run(vm, "i = 5 ; while i > 0 ; println(i) ; i = i - 1 ; end");

    crate::println!("3) Ham de quy tinh giai thua:");
    let _ = compiler::jl_run(vm,
        "function fact(n) ; if n == 1 ; return 1 ; end ; return n * fact(n - 1) ; end ; println(fact(6))"
    );
}

/// Chạy tất cả demos
pub fn run_all_demos(vm: &mut ForthVm) {
    crate::println!("======================================================");
    crate::println!("  FORTHVM -- May Ao Julia-Forth tren MyKernel");
    crate::println!("  Phase 25: Userland Runtime");
    crate::println!("======================================================");
    crate::println!("");

    demo_phase1(vm);
    crate::println!("");
    demo_phase2(vm);
    crate::println!("");
    demo_phase3(vm);
    crate::println!("");
    demo_phase5(vm);
    crate::println!("");
    demo_phase6(vm);
    crate::println!("");
    demo_phase8(vm);
    crate::println!("");

    crate::println!("======================================================");
    crate::println!("  Tat ca demos hoan thanh!");
    crate::println!("  Go 'julia <code>' de viet code truc tiep.");
    crate::println!("======================================================");
}
