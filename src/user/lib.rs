#![no_std]
#![feature(linkage)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

use core::{
    arch::asm,
    fmt::{self, Write},
    panic::PanicInfo,
};
use kernel::syscall_num::{SYS_EXIT, SYS_GETCHAR, SYS_PUTCHAR};

#[inline(always)]
fn syscall(sysno: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret: isize;
    unsafe {
        // x10: a0, x11: a1, x12: a2 -> x10: system call result
        asm!(
            "ecall",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a3") sysno
        );
    }
    ret
}

pub fn put_char(ch: char) {
    syscall(SYS_PUTCHAR, ch as usize, 0, 0);
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            put_char(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print(format_args!($fmt $(, $($arg)+)?))
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

/// Read stdin
///
/// #Return
/// - get => char
/// - if get nothing => loop in kernel
pub fn get_char() -> usize {
    syscall(SYS_GETCHAR, 0, 0, 0) as usize
}

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
            stack_top = sym __stack_top, // This symbol is defined by user.ld.
            options(noreturn)
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.location() {
        Some(location) => {
            println!(
                "[user] Panicked at {}:{} {}",
                location.file(),
                location.line(),
                info.message().unwrap()
            );
        }
        None => println!("[user] Panicked: {}", info.message().unwrap()),
    };
    loop {}
}
