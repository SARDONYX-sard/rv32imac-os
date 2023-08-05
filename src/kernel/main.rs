#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(fn_align)]

use core::{arch::asm, panic::PanicInfo};

use kernel::println;

// Defined symbols by kernel.ld
extern "C" {
    fn __bss();
    fn __bss_end();
    fn __stack_top();
}

#[inline]
fn clear_bss() {
    let bss_start = __bss as usize;
    let bss_end = __bss_end as usize;
    let bss_size = bss_end as usize - bss_start as usize;
    unsafe {
        core::slice::from_raw_parts_mut(bss_start as *mut u8, bss_size).fill(0);
    }
}

fn kernel_main() {
    clear_bss();
    let s = "Hello World!";
    println!("{}", s);
    loop {
        unsafe { asm!("wfi") }
    }
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub extern "C" fn boot() {
    unsafe {
        asm!(
            "la sp, {stack_top}", // Move the stack pointer to the __stack_top address
            "call {kernel_main}",
            stack_top = sym __stack_top,
            kernel_main = sym kernel_main,
            options(noreturn)
        );
    }
}

/// NOTE: info is not used yet because println!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
