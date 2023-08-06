use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use kernel::PhysAddr;

extern "C" {
    /// 4KiB alignment by kernel.ld
    fn __free_ram();
    /// Defined  by kernel.ld
    fn __free_ram_end();
}
const PAGE_SIZE: usize = 0x1000;
static NEXT_PADDR: AtomicUsize = AtomicUsize::new(0);

fn next_paddr_init_once() {
    // NOTE: global flag variable
    static IS_NEXT_PADDR_INIT: AtomicBool = AtomicBool::new(false);

    if !IS_NEXT_PADDR_INIT.load(Ordering::Acquire) {
        NEXT_PADDR.store(__free_ram as usize, Ordering::Release);
        IS_NEXT_PADDR_INIT.store(true, Ordering::Release);
    };
}

/// n time allocate pages & 0 fill, return it's addr
pub fn alloc_pages(n: usize) -> PhysAddr {
    next_paddr_init_once();
    let paddr = NEXT_PADDR.load(Ordering::Acquire);
    NEXT_PADDR.fetch_add(n * PAGE_SIZE, Ordering::Relaxed);

    assert!(
        NEXT_PADDR.load(Ordering::Acquire) < __free_ram_end as usize,
        "out of memory"
    );

    unsafe {
        let paddr = core::slice::from_raw_parts_mut(paddr as *mut u8, n * PAGE_SIZE);
        paddr.fill(0);
        (paddr.as_ptr() as usize).into()
    }
}
