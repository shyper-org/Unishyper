use core::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::ptr::NonNull;
// rCore buddy system allocator
use buddy_system_allocator::Heap;

use crate::libs::traits::*;
use crate::libs::synch::spinlock::SpinlockIrqSave;

pub fn init() {
    // We dump the current memory layout here.
    println!("Booting, memory layout:");
    println!(
        "Heap range:\tkva [{:#x} - {:#x}] size {} KB",
        super::config::heap_range().start.pa2kva(),
        super::config::heap_range().end.pa2kva(),
        (super::config::heap_range().end - super::config::heap_range().start) / 1024
    );

    // We need to init the global heap allocator here.
    // Because during the process of paged_ranges, a vector is required.
    // See config.rs for more details.
    let range = super::config::heap_range();
    unsafe { HEAP_ALLOCATOR.init(range.start.pa2kva(), range.end - range.start) }

    let mut paged_ranges_size = 0;
    for range in super::config::paged_ranges() {
        println!(
            "Paged range:\tkva [{:#x} - {:#x}] size {} KB",
            range.start.pa2kva(),
            range.end.pa2kva(),
            (range.end - range.start) / 1024
        );
        paged_ranges_size += range.end - range.start;
    }
    println!("Total Free Memory size {} KB", paged_ranges_size / 1024);

    println!("ELF File Load at {:#x}", crate::board::ELF_IMAGE_LOAD_ADDR);
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
        println!(
            "HEAP_ALLOCATOR init range [{:#x} - {:#x}]",
            start,
            start + size
        );
        unsafe {
            self.0.lock().init(start, size);
        }
    }
}

unsafe impl GlobalAlloc for SpinlockIrqSaveHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // println!(
        //     "GlobalAlloc alloc {:?} with pkru {:#x}",
        //     layout,
        //     crate::arch::mpk::rdpkru()
        // );
        let res = self
            .0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr());
        // println!(
        //     "GlobalAlloc {} alloc success at {:#p} {:?}",
        //     crate::libs::thread::current_thread_id(),
        //     res,
        //     layout,
        // );
        res
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // println!(
        //     "GlobalAlloc {} dealloc success at {:#p} {:?}",
        //     crate::libs::thread::current_thread_id(),
        //     ptr,
        //     layout,
        // );
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[cfg(not(feature = "std"))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    println!(
        "alloc_error_handler: heap panic on Layout size {:#x} = {}KB = {}MB  align {:#x}",
        layout.size(),
        layout.size() / 1024,
        layout.size() / 1024 / 1024,
        layout.align()
    );
    let stats_total_bytes = HEAP_ALLOCATOR.0.lock().stats_total_bytes();
    let stats_alloc_user = HEAP_ALLOCATOR.0.lock().stats_alloc_user();
    let stats_alloc_actual = HEAP_ALLOCATOR.0.lock().stats_alloc_actual();
    println!(
        "alloc_error_handler STATS: total_bytes {:#x} = {}KB = {}MB alloc_user {:#x} alloc_actual {:#x}",
        stats_total_bytes,
        stats_total_bytes / 1024,
        stats_total_bytes / 1024 / 1024,
        stats_alloc_user,
        stats_alloc_actual
    );
    loop {}
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

use core::ptr;
use core::alloc::{Allocator, AllocError};

pub struct Global;

unsafe impl Allocator for Global {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
            // SAFETY: `layout` is non-zero in size,
            size => {
                let raw_ptr = HEAP_ALLOCATOR
                    .0
                    .lock()
                    .alloc(layout)
                    .ok()
                    .map_or(core::ptr::null_mut() as *mut u8, |mut mem| unsafe {
                        mem.as_mut()
                    });
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(NonNull::slice_from_raw_parts(ptr, size))
            }
        }
    }
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocate(layout)?;
        // SAFETY: `alloc` returns a valid memory block
        use crate::libs::string::memset;
        unsafe {
            memset(ptr.as_mut_ptr(), 0, ptr.len());
        }
        // unsafe { ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len()); }
        Ok(ptr)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // SAFETY: `layout` is non-zero in size,
            // other conditions must be upheld by the caller
            HEAP_ALLOCATOR.0.lock().dealloc(
                unsafe { core::ptr::NonNull::new_unchecked(ptr.as_ptr()) },
                layout,
            );
        }
    }
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate_zeroed(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        let new_ptr = self.allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be lower than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `new_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_layout.size());
            self.deallocate(ptr, old_layout);
        }

        Ok(new_ptr)
    }
    #[inline(always)]
    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}
