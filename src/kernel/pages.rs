use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel::addr::{is_aligned, PhysAddr, PhysPageNum, VirtAddr};

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
///
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
///
/// # Panics
/// - vaddr & paddr must be aligned to PAGE_SIZE(default: 4096)
fn map_page(root_ppn: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: usize) {
    let page_table_ptr: usize = root_ppn.into();
    let vaddr: usize = vaddr.into();
    assert!(is_aligned(vaddr, PAGE_SIZE), "unaligned vaddr {vaddr:x}");
    assert!(
        is_aligned(usize::from(paddr), PAGE_SIZE),
        "unaligned paddr {paddr}"
    );

    let table1 = unsafe {
        core::slice::from_raw_parts_mut(page_table_ptr as *mut PageTableEntry, PAGE_TABLE_LEN)
    };
    let vpn1 = (vaddr >> 22) & (PAGE_TABLE_LEN - 1); // usize bit - (VPN[0](10) + offset(12)) = 10bit
    if !table1[vpn1].is_valid() {
        let pt_paddr: PhysPageNum = alloc_pages(1).into();
        table1[vpn1] = PageTableEntry(usize::from(pt_paddr) << 10 | PAGE_V);
    }

    let table0_ptr = (table1[vpn1].0 >> 10) * PAGE_SIZE;
    let vpn0 = (vaddr >> 12) & (1024 - 1);
    let table0 = unsafe {
        core::slice::from_raw_parts_mut(table0_ptr as *mut PageTableEntry, PAGE_TABLE_LEN)
    };
    table0[vpn0] = PageTableEntry(((usize::from(PhysPageNum::from(paddr))) << 10) | flags | PAGE_V);
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

const MAX_APP_NUM: usize = 16;
/// USER Application start address
pub const USER_BASE: usize = 0x1000000;

/// Get .text section in kernel application address list.
pub fn get_user_app_list() -> [(usize, usize); MAX_APP_NUM] {
    extern "C" {
        fn _num_app();
    }
    // Expected that following tuple array.
    //  (.word app_0_start ptr, .word app_0_end ptr)
    let mut apps_ptr_list: [(usize, usize); MAX_APP_NUM] = [(0, 0); MAX_APP_NUM];
    let num_app_ptr = _num_app as usize as *const usize;

    // start app_0_start ptr
    let mut app_n_ptr = num_app_ptr;
    // read_volatile: read ptr content
    // _num_app:
    //       .word 1 <- I want to get this value.
    let num_app = unsafe { num_app_ptr.read_volatile() };
    for app_n in apps_ptr_list.iter_mut().take(num_app) {
        let app_n_start_ptr = unsafe {
            app_n_ptr = app_n_ptr.add(1);
            app_n_ptr.read_volatile()
        };
        let app_n_end_ptr = unsafe {
            app_n_ptr = app_n_ptr.add(1);
            app_n_ptr.read_volatile()
        };
        *app_n = (app_n_start_ptr, app_n_end_ptr);
    }

    apps_ptr_list
}

/// Allocate page with user-authorized apps.
///
/// # Parameters
/// - root_ppn: proc root node(satp)
/// - app_start(end)_ptr: phys addr in kernel text section app ptr
pub fn map_one_app(root_ppn: usize, app_start_ptr: usize, app_end_ptr: usize) {
    let app_size = app_end_ptr - app_start_ptr;
    // each page offset index
    let mut offset = 0;
    while offset < app_size {
        let page: usize = alloc_pages(1).into();
        // Copy the user application embedded in the text section of the kernel to the page allocated on a page-by-page basis
        unsafe {
            let app_dst = core::slice::from_raw_parts_mut(page as *mut u8, PAGE_SIZE);
            let app_src =
                core::slice::from_raw_parts((app_start_ptr + offset) as *const u8, PAGE_SIZE);
            app_dst.copy_from_slice(app_src);
        }
        // Associate USER_BASE (base address of virtual address) with the allocated page
        map_page(
            root_ppn.into(),
            (USER_BASE + offset).into(),
            page.into(),
            PAGE_U | PAGE_R | PAGE_W | PAGE_X,
        );
        offset += PAGE_SIZE
    }
}
