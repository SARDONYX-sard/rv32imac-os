#![no_std]
#![no_main]

extern crate user_lib;

#[no_mangle]
pub fn main() {
    // unsafe { (0x80200000 as *const u8).read_volatile() }; // attempt kernel address. expected load page fault.
    #[allow(clippy::empty_loop)]
    loop {}
}
