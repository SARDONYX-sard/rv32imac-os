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

/// # Physical address(SV32: 32bit)
///
/// | BitNum  |32----------------12|11---------0|
/// |---------|--------------------|------------|
/// | Meaning | PhysicalPageNumber | PageOffset |
/// |  Width  |         20         |     12     |
#[derive(Clone, Copy, Debug)]
pub struct PhysAddr(usize);
/// # Physical Number(SV32: 20bit)
///
/// | BitNum  |32----------------12|11---------0|
/// |---------|--------------------|------------|
/// | Meaning | PhysicalPageNumber | PageOffset |
/// |  Width  |         20         |     12     |
#[derive(Clone, Copy, Debug)]
pub struct PhysPageNum(usize);

/// # Virtual Number(SV32: 20bit)
///
/// | Meaning |  VPN1   |   VPN0   |            |
/// |---------|---------|----------|----------- |
/// |  Width  |    9    |     9    |            |
/// | BitNum  |38----------------12|11---------0|
/// |---------|--------------------|------------|
/// | Meaning | VirtualPageNumber  | PageOffset |
/// |  Width  |         27         |     12     |
#[derive(Clone, Copy, Debug)]
pub struct VirtPageNum(usize);
/// Virtual Address
///
/// | Meaning |  VPN1   |   VPN0   |            |
/// |---------|---------|----------|----------- |
/// |  Width  |    9    |     9    |            |
/// | BitNum  |38----------------12|11---------0|
/// |---------|--------------------|------------|
/// | Meaning | VirtualPageNumber  | PageOffset |
/// |  Width  |         27         |     12     |
#[derive(Clone, Copy, Debug)]
pub struct VirtAddr(usize);

const PAGE_OFFSET: usize = 12;
macro_rules! impl_page_num_from {
    ($id:ident -> $page_num:ident) => {
        impl From<$id> for $page_num {
            fn from(addr: $id) -> Self {
                Self(addr.0 >> PAGE_OFFSET)
            }
        }
        impl From<$page_num> for $id {
            fn from(num_addr: $page_num) -> Self {
                Self(num_addr.0 << PAGE_OFFSET)
            }
        }
    };
}
impl_page_num_from!(PhysAddr -> PhysPageNum);
impl_page_num_from!(VirtAddr -> VirtPageNum);

macro_rules! impl_addr_utility {
    ($($id:ident),+ $(,)?) => {
        $(
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
        )+
    };
}
impl_addr_utility!(PhysAddr, VirtAddr, PhysPageNum, VirtPageNum);
