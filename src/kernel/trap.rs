use core::arch::asm;
use kernel::riscv::{
    scause::{self, Scause},
    Sepc, Stval,
};

#[repr(C)]
pub struct TrapFrame {
    ra: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    sp: usize,
}

/// Save register & jump to trap(Systemcall, interrupt, etc.) event handler.
#[naked] // Use this attribute to manually control the stack so that no extra code is output.
#[repr(align(4))] // Set the least significant 2 bits to 0 for mode flag.
pub extern "C" fn kernel_entry() {
    // sscratch registers: registers that the kernel is free to use
    unsafe {
        asm!(
            // Extract the kernel stack of the running process from sscratch
            "csrrw sp, sscratch, sp", // atomic swap sscratch <-> sp

            // - Why multiply 4byte? => 32bit RISC-V(RV32). 32bit == 4byte register
            // - This OS also uses an addition instruction,
            //   although the stack grows in the direction of smaller addresses.
            //   However, since the immediate value being added is negative,
            //   it eventually grows to a smaller address as usual.

            // --- create TrapFrame array
            "addi sp, sp, -4 * 31", // allocate stack
            "sw ra,  4 * 0(sp)",    // Memory[sp + 0 * 4] = ra
            "sw gp,  4 * 1(sp)",
            // temporary registers
            "sw tp,  4 * 2(sp)",
            "sw t0,  4 * 3(sp)",
            "sw t1,  4 * 4(sp)",
            "sw t2,  4 * 5(sp)",
            "sw t3,  4 * 6(sp)",
            "sw t4,  4 * 7(sp)",
            "sw t5,  4 * 8(sp)",
            "sw t6,  4 * 9(sp)",
            // argument registers
            "sw a0,  4 * 10(sp)",
            "sw a1,  4 * 11(sp)",
            "sw a2,  4 * 12(sp)",
            "sw a3,  4 * 13(sp)",
            "sw a4,  4 * 14(sp)",
            "sw a5,  4 * 15(sp)",
            "sw a6,  4 * 16(sp)",
            "sw a7,  4 * 17(sp)",
            // callee saved registers
            "sw s0,  4 * 18(sp)",
            "sw s1,  4 * 19(sp)",
            "sw s2,  4 * 20(sp)",
            "sw s3,  4 * 21(sp)",
            "sw s4,  4 * 22(sp)",
            "sw s5,  4 * 23(sp)",
            "sw s6,  4 * 24(sp)",
            "sw s7,  4 * 25(sp)",
            "sw s8,  4 * 26(sp)",
            "sw s9,  4 * 27(sp)",
            "sw s10, 4 * 28(sp)",
            "sw s11, 4 * 29(sp)",

            "csrr a0, sscratch", // a0 = sscratch: exception occurred sp to a0
            "sw a0, 4 * 30(sp)", // Trapframe sp field = a0

            "addi a0, sp, 4 * 31", // a0 = stack start address
            "csrw sscratch, a0", // sscratch = stack start address
            // --- create TrapFrame array end

            "mv a0, sp", // a0 = stack current address
            "call {trap_handler}",

            "lw ra,  4 * 0(sp)",
            "lw gp,  4 * 1(sp)",
            "lw tp,  4 * 2(sp)",
            "lw t0,  4 * 3(sp)",
            "lw t1,  4 * 4(sp)",
            "lw t2,  4 * 5(sp)",
            "lw t3,  4 * 6(sp)",
            "lw t4,  4 * 7(sp)",
            "lw t5,  4 * 8(sp)",
            "lw t6,  4 * 9(sp)",
            "lw a0,  4 * 10(sp)",
            "lw a1,  4 * 11(sp)",
            "lw a2,  4 * 12(sp)",
            "lw a3,  4 * 13(sp)",
            "lw a4,  4 * 14(sp)",
            "lw a5,  4 * 15(sp)",
            "lw a6,  4 * 16(sp)",
            "lw a7,  4 * 17(sp)",
            "lw s0,  4 * 18(sp)",
            "lw s1,  4 * 19(sp)",
            "lw s2,  4 * 20(sp)",
            "lw s3,  4 * 21(sp)",
            "lw s4,  4 * 22(sp)",
            "lw s5,  4 * 23(sp)",
            "lw s6,  4 * 24(sp)",
            "lw s7,  4 * 25(sp)",
            "lw s8,  4 * 26(sp)",
            "lw s9,  4 * 27(sp)",
            "lw s10, 4 * 28(sp)",
            "lw s11, 4 * 29(sp)",
            "lw sp,  4 * 30(sp)",
            "sret",
            trap_handler = sym handle_trap,
            options(noreturn),
        )
    }
}

#[no_mangle]
fn handle_trap(_f: TrapFrame) {
    let scause: Scause = unsafe { scause::read() }.into();
    let stval = unsafe { Stval::read() };
    let user_pc = unsafe { Sepc::read() };

    panic!(
        "unexpected trap scause={:?}, stval={:x}, sepc={:x}",
        scause, stval, user_pc,
    );
}
