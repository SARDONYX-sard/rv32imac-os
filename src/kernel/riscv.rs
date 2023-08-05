use core::arch::asm;

#[derive(Copy, Clone, Debug)]
pub struct Scause(usize);

impl Scause {
    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        asm!("csrr {}, scause", out(reg) value);
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("csrw scause, {}", in(reg) value);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stval(usize);

impl Stval {
    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        asm!("csrr {}, stval", out(reg) value);
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("csrw stval, {}", in(reg) value);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sepc(usize);

impl Sepc {
    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        asm!("csrr {}, sepc", out(reg) value);
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("csrw sepc, {}", in(reg) value);
    }
}

pub mod stvec {
    use core::arch::asm;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum TrapMode {
        Direct = 0,
        Vectored = 1,
    }

    #[inline]
    pub unsafe fn write(addr: usize, mode: TrapMode) {
        asm!("csrw stvec, {}", in(reg) addr | mode as usize);
    }
}
