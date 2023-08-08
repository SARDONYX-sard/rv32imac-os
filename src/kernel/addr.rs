use core::fmt;

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub fn align_up(value: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (value + (align - 1)) & !(align - 1)
}

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub fn align_down(value: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    value & !(align - 1)
}

/// # Parameters
/// - value:
/// - align: expected power of two
///
/// # Panics
/// align not power of two
pub fn is_aligned(value: usize, align: usize) -> bool {
    assert!(align.is_power_of_two());
    value & (align - 1) == 0
}

/// Physical Address
pub struct PhysAddr(usize);
/// Virtual Address
///
/// | Meaning |  VPN1   |   VPN0   |            |
/// |---------|---------|----------|----------- |
/// |  Width  |    9    |     9    |            |
/// | BitNum  |38----------------12|11---------0|
/// |---------|--------------------|------------|
/// | Meaning | VirtualPageNumber  | PageOffset |
/// |  Width  |         27         |     12     |
pub struct VirtAddr(usize);

macro_rules! impl_addr_utility {
    ($id:ident) => {
        impl From<usize> for $id {
            fn from(addr: usize) -> Self {
                $id(addr)
            }
        }

        impl From<$id> for usize {
            fn from(addr: $id) -> Self {
                addr.0
            }
        }

        impl fmt::Display for $id {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($id), " = {:x}"), self.0)
            }
        }
    };
}

impl_addr_utility!(PhysAddr);
impl_addr_utility!(VirtAddr);
