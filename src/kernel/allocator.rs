use crate::pages::alloc_pages;
use core::alloc::{GlobalAlloc, Layout};

struct BumpPtrAlloc {}

impl BumpPtrAlloc {
    const fn empty() -> Self {
        Self {}
    }
}

unsafe impl Sync for BumpPtrAlloc {}

unsafe impl GlobalAlloc for BumpPtrAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let paddr: usize = alloc_pages(layout.size()).into();
        paddr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static SIMPLE_ALLOCATOR: BumpPtrAlloc = BumpPtrAlloc::empty();
#[alloc_error_handler]
fn hlt(_layout: Layout) -> ! {
    panic!("Failed to allocate heap")
}
