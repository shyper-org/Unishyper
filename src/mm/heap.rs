use core::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::ptr::NonNull;
// rCore buddy system allocator
use buddy_system_allocator::Heap;

use crate::libs::traits::*;
use crate::libs::synch::spinlock::SpinlockIrqSave;

pub fn init() {
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
        super::config::heap_range().start,
        super::config::heap_range().end,
        super::config::heap_range().start.pa2kva(),
        super::config::heap_range().end.pa2kva()
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
    );

    let range = super::config::heap_range();
    unsafe { HEAP_ALLOCATOR.init(range.start.pa2kva(), range.end - range.start) }
}

#[cfg(feature = "terminal")]
pub fn dump_heap_allocator_state() {
    let lock = HEAP_ALLOCATOR.0.lock();
    let alloc_actual = lock.stats_alloc_actual();
    let alloc_user = lock.stats_alloc_user();
    let alloc_total = lock.stats_total_bytes();
    println!("Buddy system heap allocator, total: {} Bytes", alloc_total);
    println!(
        "Allocated user: {} Bytes, actual: {} Bytes",
        alloc_user, alloc_actual
    );
}

struct SpinlockIrqSaveHeapAllocator(SpinlockIrqSave<Heap<32>>);

#[global_allocator]
static HEAP_ALLOCATOR: SpinlockIrqSaveHeapAllocator = SpinlockIrqSaveHeapAllocator::empty();

impl SpinlockIrqSaveHeapAllocator {
    /// Create an empty heap.
    pub const fn empty() -> SpinlockIrqSaveHeapAllocator {
        SpinlockIrqSaveHeapAllocator(SpinlockIrqSave::new(Heap::empty()))
    }

    /// Add a range of memory [start, end) to the heap.
    pub unsafe fn init(&self, start: usize, size: usize) {
        unsafe {
            self.0.lock().init(start, size);
        }
    }
}

unsafe impl GlobalAlloc for SpinlockIrqSaveHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[cfg(not(feature = "std"))]
#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
    panic!("alloc_error_handler: heap panic");
}

/// Interface to allocate memory from system heap.
///
/// # Errors
/// Returning a null pointer indicates that either memory is exhausted or
/// `size` and `align` do not meet this allocator's size or alignment constraints.
#[cfg(feature = "std")]
pub fn malloc(size: usize, align: usize) -> *mut u8 {
    let layout_res = Layout::from_size_align(size, align);
    if layout_res.is_err() || size == 0 {
        warn!(
            "heap malloc called with size {:#x}, align {:#x} is an invalid layout!",
            size, align
        );
        return core::ptr::null::<*mut u8>() as *mut u8;
    }
    let layout = layout_res.unwrap();
    let ptr = HEAP_ALLOCATOR
        .0
        .lock()
        .alloc(layout)
        .ok()
        .map_or(core::ptr::null_mut() as *mut u8, |mut mem| unsafe {
            mem.as_mut()
        });

    trace!(
        "heap malloc: allocate memory at {:#x} (size {:#x}, align {:#x})",
        ptr as usize,
        size,
        align
    );

    ptr
}

/// Interface to deallocate a memory region from the system heap
///
/// # Safety
/// This function is unsafe because undefined behavior can result if the caller does not ensure all of the following:
/// - ptr must denote a block of memory currently allocated via this allocator,
/// - `size` and `align` must be the same values that were used to allocate that block of memory
/// TODO: verify if the same values for size and align always lead to the same layout
///
/// # Errors
/// May panic if debug assertions are enabled and invalid parameters `size` or `align` where passed.
#[cfg(feature = "std")]
pub fn free(ptr: *mut u8, size: usize, align: usize) {
    let layout_res = Layout::from_size_align(size, align);
    if layout_res.is_err() || size == 0 {
        warn!(
            "heap free called with size {:#x}, align {:#x} is an invalid layout!",
            size, align
        );
        debug_assert!(layout_res.is_err(), "heap free error: Invalid layout");
        debug_assert_ne!(size, 0, "heap free error: size cannot be 0");
    } else {
        trace!(
            "heap free: deallocate memory at {:#x} (size {:#x})",
            ptr as usize,
            size
        );
    }
    let layout = layout_res.unwrap();
    HEAP_ALLOCATOR
        .0
        .lock()
        .dealloc(unsafe { core::ptr::NonNull::new_unchecked(ptr) }, layout);
}
