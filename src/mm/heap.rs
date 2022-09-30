use core::alloc::Layout;

// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

use crate::libs::traits::*;

pub fn init() {
    let range = super::config::heap_range();
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(range.start.pa2kva(), range.end - range.start)
    }
    println!(
        "Heap range: pa [{:x} - {:x}] va [{:x} - {:x}]",
        range.start,
        range.end,
        range.start.pa2kva(),
        range.end.pa2kva()
    )
}

#[cfg(feature = "terminal")]
pub fn dump_heap_allocator_state() {
    let lock = HEAP_ALLOCATOR.lock();
    let alloc_actual = lock.stats_alloc_actual();
    let alloc_user = lock.stats_alloc_user();
    let alloc_total = lock.stats_total_bytes();
    println!("Buddy system heap allocator, total: {} Bytes", alloc_total);
    println!(
        "Allocated user: {} Bytes, actual: {} Bytes",
        alloc_user, alloc_actual
    );
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
    panic!("alloc_error_handler: heap panic");
}
