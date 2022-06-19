use core::alloc::Layout;

// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

use crate::lib::traits::*;

pub fn init() {
    let range = super::config::heap_range();
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(range.start.pa2kva(), range.end - range.start)
    }
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
    panic!("alloc_error_handler: heap panic");
}
