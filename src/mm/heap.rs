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
    println!("Booting, memory layout:");
    println!(
        "Kernel range:\tpa [{:x} - {:x}] kva [{:x} - {:x}]",
        super::config::kernel_range().start,
        super::config::kernel_range().end,
        super::config::kernel_range().start.pa2kva(),
        super::config::kernel_range().end.pa2kva()
    );
    println!(
        "Heap range:\tpa [{:x} - {:x}] kva [{:x} - {:x}]",
        range.start,
        range.end,
        range.start.pa2kva(),
        range.end.pa2kva()
    );
    println!(
        "ELF range:\tpa [{:x} - {:x}] kva [{:x} - {:x}]",
        super::config::elf_range().start,
        super::config::elf_range().end,
        super::config::elf_range().start.pa2kva(),
        super::config::elf_range().end.pa2kva()
    );
    println!(
        "Paged range:\tpa [{:x} - {:x}] kva [{:x} - {:x}]",
        super::config::paged_range().start,
        super::config::paged_range().end,
        super::config::paged_range().start.pa2kva(),
        super::config::paged_range().end.pa2kva()
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
