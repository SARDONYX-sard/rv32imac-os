#![no_std]
#![feature(linkage)]
#![feature(naked_functions)]

use core::{arch::asm, panic::PanicInfo};

// const SYS_PUTCHAR: usize = 1;
// const SYS_GETCHAR: usize = 2;
// const SYS_READFILE: usize = 3;
// const SYS_WRITEFILE: usize = 4;
const SYS_EXIT: usize = 5;

#[inline(always)]
fn syscall(sysno: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let mut ret: isize;
    unsafe {
        // x10: a0, x11: a1, x12: a2 -> x10: system call result
        asm!(
            "ecall",
            inlateout("x10") arg1 => ret,
            in("x11") arg2,
            in("x12") arg3,
            in("x17") sysno
        );
    }
    ret
}

// pub fn putchar(ch: char) {
//     syscall(SYS_PUTCHAR, ch as usize, 0, 0);
// }

// pub fn getchar() -> isize {
//     syscall(SYS_GETCHAR, 0, 0, 0)
// }

// pub fn readfile(filename: &str, buf: &mut [u8]) -> isize {
//     let filename_ptr = filename.as_ptr() as usize;
//     let buf_ptr = buf.as_mut_ptr() as usize;
//     let len = buf.len();
//     syscall(SYS_READFILE, filename_ptr, buf_ptr, len)
// }

// pub fn writefile(filename: &str, buf: &[u8]) -> isize {
//     let filename_ptr = filename.as_ptr() as usize;
//     let buf_ptr = buf.as_ptr() as usize;
//     let len = buf.len();
//     syscall(SYS_WRITEFILE, filename_ptr, buf_ptr, len)
// }

#[no_mangle]
pub extern "C" fn exit() {
    syscall(SYS_EXIT, 0, 0, 0);
}

#[no_mangle]
#[naked]
#[link_section = ".text.start"]
pub extern "C" fn start() {
    extern "C" {
        /// This symbol is defined by user.ld.
        fn __stack_top();
    }

    unsafe {
        asm!(
            "la sp, {stack_top}",
            "call main",
            "call exit",
            stack_top = sym __stack_top, // This symbol is defined by user.ld.
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // match info.location() {
    //     Some(location) => {
    //         println!(
    //             "[user] Panicked at {}:{} {}",
    //             location.file(),
    //             location.line(),
    //             info.message().unwrap()
    //         );
    //     }
    //     None => println!("[user] Panicked: {}", info.message().unwrap()),
    // };
    loop {}
}
