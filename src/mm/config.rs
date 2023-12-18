use core::ops::Range;
use alloc::vec::Vec;

use spin::Once;

use crate::arch::PAGE_SIZE;
pub const STACK_SIZE: usize = PAGE_SIZE * 512;
// pub const STACK_SIZE: usize = PAGE_SIZE * 64;
// pub const STACK_SIZE: usize = PAGE_SIZE * 2;

pub fn heap_range() -> Range<usize> {
    use crate::board::GLOBAL_HEAP_SIZE;
    use crate::arch::MACHINE_SIZE;
    const HEAP_BLOCK: usize = GLOBAL_HEAP_SIZE / MACHINE_SIZE;
    static mut HEAP: [usize; HEAP_BLOCK] = [0; HEAP_BLOCK];
    unsafe { HEAP.as_ptr() as usize..HEAP.as_ptr() as usize + HEAP_BLOCK * MACHINE_SIZE }
}

#[cfg(not(target_arch = "x86_64"))]
use crate::libs::traits::*;

#[cfg(not(target_arch = "x86_64"))]
use crate::mm::address::VAddr;

#[cfg(not(target_arch = "x86_64"))]
pub fn kernel_end_address() -> VAddr {
    extern "C" {
        // Note: link-time label, see linker.ld
        fn KERNEL_END();
    }
    use crate::util::round_up;
    VAddr::new_canonical(round_up(
        (KERNEL_END as usize).kva2pa(),
        crate::arch::PAGE_SIZE,
    ))
}

static FRAME_RANGES: Once<Vec<Range<usize>>> = Once::new();

#[cfg(not(target_arch = "x86_64"))]
pub fn paged_ranges() -> &'static Vec<Range<usize>> {
    match FRAME_RANGES.get() {
        None => FRAME_RANGES.call_once(|| {
            let mut frame_ranges = Vec::new();
            #[cfg(not(feature = "k210"))]
            frame_ranges.push(kernel_end_address().value()..crate::board::ELF_IMAGE_LOAD_ADDR);
            #[cfg(feature = "k210")]
            frame_ranges
                .push(kernel_end_address().value()..crate::board::BOARD_NORMAL_MEMORY_RANGE.end);

            frame_ranges
        }),
        Some(x) => x,
    }
}

#[cfg(target_arch = "x86_64")]
pub fn paged_ranges() -> &'static Vec<Range<usize>> {
    use rboot::MemoryType;
    match FRAME_RANGES.get() {
        None => FRAME_RANGES.call_once(|| {
            let mut frame_ranges = Vec::new();
            for (_idx, region) in crate::arch::boot_info()
                .memory_map
                .into_iter()
                .filter(|region| region.ty == MemoryType::CONVENTIONAL)
                .enumerate()
            {
                let start = region.phys_start as usize;
                let end = region.phys_start as usize
                    + region.page_count as usize * crate::arch::PAGE_SIZE;
                frame_ranges.push(start..end)
            }
            frame_ranges
        }),
        Some(x) => x,
    }
}
