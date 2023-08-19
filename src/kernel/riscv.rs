#![allow(clippy::missing_safety_doc)]

use core::arch::asm;

/// - see instruction explation(https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md#instruction-aliases)
pub fn raise_exception() {
    unsafe { asm!("unimp") }
}

pub mod scause {
    use core::arch::asm;

    /// Human Readable scause
    /// - see [Scause 4.1.9](https://people.eecs.berkeley.edu/~krste/papers/riscv-privileged-v1.9.pdf)
    #[derive(Debug)]
    pub enum Scause {
        Interrupt(Interrupt),
        Exception(Exception),
    }

    impl From<usize> for Scause {
        fn from(value: usize) -> Self {
            // We want to check if the most significant bit (32nd bit if 32 bits), which is the interrupt flag, is 1.
            let is_interrupt = (value & 1 >> (core::mem::size_of::<usize>() - 1)) == 1;
            match is_interrupt {
                true => Scause::Interrupt(Interrupt::try_from(value).unwrap_or_default()),
                false => Scause::Exception(Exception::try_from(value).unwrap_or_default()),
            }
        }
    }

    #[derive(Debug, Default)]
    pub enum Interrupt {
        UserSoftware,
        SupervisorSoftware,
        UserTimer,
        SupervisorTimer,
        #[default]
        Unknown,
    }

    impl TryFrom<usize> for Interrupt {
        type Error = Interrupt;
        fn try_from(value: usize) -> Result<Self, Self::Error> {
            // Clears the most significant bit (interrupt flag) to convert to enum in match expression.
            let value = value & (1 << (core::mem::size_of::<usize>() - 1));
            Ok(match value {
                0 => Interrupt::UserSoftware,
                1 => Interrupt::SupervisorSoftware,
                4 => Interrupt::UserTimer,
                5 => Interrupt::SupervisorTimer,
                _ => return Err(Interrupt::Unknown),
            })
        }
    }

    #[derive(Debug, Default)]
    pub enum Exception {
        InstructionAddressMisaligned,
        InstructionAccessFault,
        IllegalInstruction,
        Breakpoint,
        LoadAccessFault,
        /// AMO(Atomic memory operation)
        AmoAddressMisaligned,
        /// Store AMO(Atomic memory operation) fault
        StoreAmoAccessFault,
        EnvironmentCall,
        InstructionPageFault,
        LoadpageFault,
        StoreAmoPageFault,
        #[default]
        Unknown,
    }

    impl TryFrom<usize> for Exception {
        type Error = Exception;
        fn try_from(value: usize) -> Result<Self, Self::Error> {
            Ok(match value {
                0 => Exception::InstructionAddressMisaligned,
                1 => Exception::InstructionAccessFault,
                2 => Exception::IllegalInstruction,
                3 => Exception::Breakpoint,
                5 => Exception::LoadAccessFault,
                6 => Exception::AmoAddressMisaligned,
                7 => Exception::StoreAmoAccessFault,
                8 => Exception::EnvironmentCall,
                12 => Exception::InstructionPageFault,
                13 => Exception::LoadpageFault,
                15 => Exception::StoreAmoPageFault,
                _ => return Err(Exception::Unknown),
            })
        }
    }

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

pub mod sepc {
    use core::arch::asm;

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

pub mod sstatus {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> usize {
        let mut value;
        asm!("csrr {}, sstatus", out(reg) value);
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("csrw sstatus, {}", in(reg) value);
    }
}
