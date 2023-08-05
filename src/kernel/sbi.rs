use core::arch::asm;

#[allow(unused)]
pub(crate) struct SbiRet {
    error: usize,
    value: usize,
}

#[inline(always)]
pub(crate) fn sbi_call(args: [usize; 8]) -> SbiRet {
    let mut error;
    let mut value;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => error,
            inlateout("a1") args[1] => value,
            in("a2") args[2],
            in("a3") args[3],
            in("a4") args[4],
            in("a5") args[5],
            in("a6") args[6],
            in("a7") args[7],
        );
    }

    SbiRet { error, value }
}
