use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel::addr::VirtAddr;
use kernel::addr::{is_aligned, PhysAddr};

extern "C" {
    /// defined by kernel.ld
    fn __kernel_base();
    /// 4KiB alignment by kernel.ld
    fn __free_ram();
    /// Defined  by kernel.ld
    fn __free_ram_end();
}

/// n time allocate pages & 0 fill, return it's addr
pub fn alloc_pages(n: usize) -> PhysAddr {
    static NEXT_PADDR: AtomicUsize = AtomicUsize::new(0);
    fn next_paddr_init_once() {
        // NOTE: global flag variable
        static IS_NEXT_PADDR_INIT: AtomicBool = AtomicBool::new(false);

        if !IS_NEXT_PADDR_INIT.load(Ordering::Acquire) {
            NEXT_PADDR.store(__free_ram as usize, Ordering::Release);
            IS_NEXT_PADDR_INIT.store(true, Ordering::Release);
        };
    }

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

pub const PAGE_SIZE: usize = 0x1000;
/// The number of page table entries in SV32 consists of 2^10, each of which is specified as 4 bytes.
/// - ref: https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html#sec:sv32
const PAGE_TABLE_LEN: usize = 1024;
/// Virtual page enable flag in satp(Supervisor address and protection) register
pub const SATP_SV32: usize = 1 << 31;
/// valid flag bit
const PAGE_V: usize = 1 << 0;
/// can read flag bit
const PAGE_R: usize = 1 << 1;
/// can write flag bit
const PAGE_W: usize = 1 << 2;
/// executable flag bit
const PAGE_X: usize = 1 << 3;
#[allow(unused)]
/// can access on user mode flag bit
const PAGE_U: usize = 1 << 4;

/// 4byte
/// | Bit number  |31------20|19------10|9---8| 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |-------------|----------|----------|-----|---|---|---|---|---|---|---|---|
/// | Bit meaning | PPN\[1\] | PPN\[0\] | RSW | D | A | G | U | X | W | R | V |
/// | Bit width   |    10    |    10    |  2  | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
/// - ref: https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html#sec:sv32
struct PageTableEntry(usize);

impl PageTableEntry {
    fn is_valid(&self) -> bool {
        (self.0 & PAGE_V) != 0
    }
}

/// # Parameters
/// - root_ppn: each process root node
/// - vaddr: Virtual address
/// - paddr: Physical address
/// - flags: Page table entry flags
fn map_page(root_ppn: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: usize) {
    let page_table_ptr: usize = root_ppn.into();
    let vaddr: usize = vaddr.into();
    let paddr: usize = paddr.into();
    assert!(is_aligned(vaddr, PAGE_SIZE), "unaligned vaddr {vaddr:x}");
    assert!(is_aligned(paddr, PAGE_SIZE), "unaligned paddr {paddr:x}");

    let table1 = unsafe {
        core::slice::from_raw_parts_mut(page_table_ptr as *mut PageTableEntry, PAGE_TABLE_LEN)
    };
    let vpn1 = (vaddr >> 22) & (PAGE_TABLE_LEN - 1); // usize bit - (VPN[0](10) + offset(12)) = 10bit
    if !table1[vpn1].is_valid() {
        let pt_paddr: usize = alloc_pages(1).into();
        table1[vpn1] = PageTableEntry((pt_paddr / PAGE_SIZE) << 10 | PAGE_V);
    }

    let table0_ptr = (table1[vpn1].0 >> 10) * PAGE_SIZE;
    let vpn0 = (vaddr >> 12) & (1024 - 1);
    let table0 = unsafe {
        core::slice::from_raw_parts_mut(table0_ptr as *mut PageTableEntry, PAGE_TABLE_LEN)
    };
    table0[vpn0] = PageTableEntry(((paddr / PAGE_SIZE) << 10) | flags | PAGE_V);
}

pub fn ident_map_in_kernel(root_ppn: usize) {
    let mut paddr = __kernel_base as usize;
    while paddr < __free_ram_end as usize {
        map_page(
            root_ppn.into(),
            paddr.into(),
            paddr.into(),
            PAGE_R | PAGE_W | PAGE_X,
        );
        paddr += PAGE_SIZE
    }
}
