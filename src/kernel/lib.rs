#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(fn_align)]

pub mod console;
pub mod sbi;

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub(crate) fn align_up(value: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (value + (align - 1)) & !(align - 1)
}

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub(crate) fn align_down(value: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    value & !(align - 1)
}

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub(crate) fn is_aligned(value: usize, align: usize) -> bool {
    assert!(align.is_power_of_two());
    value & (align - 1) == 0
}

/// Physical Address
pub(crate) struct PhysAddr(usize);

/// Virtual Address
pub(crate) struct VirtAddr(usize);
