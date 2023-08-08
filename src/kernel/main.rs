#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(fn_align)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
pub mod pages;
pub mod proc;
pub mod trap;

use crate::{
    proc::{run_next_proc, Executer},
    trap::kernel_entry,
};
use core::{arch::asm, panic::PanicInfo};
use kernel::{print, println, riscv::stvec};

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
    unsafe { stvec::write(kernel_entry as usize, stvec::TrapMode::Direct) };

    let mut proc_runner = Executer::new();
    proc_runner.push(|| {
        println!("starting process. A");
        let mut i = 0;
        while i <= 10 {
            print!("A");
            run_next_proc();
            i += 1;
        }
    });
    proc_runner.push(|| {
        println!("starting process. B");
        let mut i = 0;
        while i <= 10 {
            print!("B");
            run_next_proc();
            i += 1;
        }
    });
    proc_runner.run();
    println!("");
    panic!("booted!")
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
fn panic(info: &PanicInfo) -> ! {
    match info.location() {
        Some(location) => {
            println!(
                "[kernel] Panicked at {}:{} {}",
                location.file(),
                location.line(),
                info.message().unwrap()
            );
        }
        None => println!("[kernel] Panicked: {}", info.message().unwrap()),
    };
    loop {}
}
