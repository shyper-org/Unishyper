use core::ops::Range;

use crate::arch::PAGE_SIZE;
use crate::libs::traits::*;
use crate::util::round_up;

use crate::mm::address::VAddr;

pub fn kernel_end_address() -> VAddr {
    extern "C" {
        // Note: link-time label, see linker.ld
        fn KERNEL_END();
    }
    VAddr::new_canonical(round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE))
}

#[allow(unused)]
pub fn kernel_range() -> Range<usize> {
    let normal_range = crate::arch::BOARD_KERNEL_MEMORY_RANGE;
    normal_range.start..kernel_end_address().value()
}

pub fn heap_range() -> Range<usize> {
    let normal_range = crate::arch::BOARD_KERNEL_MEMORY_RANGE;
    kernel_end_address().value()..normal_range.end
}

pub fn paged_range() -> Range<usize> {
    crate::arch::BOARD_NORMAL_MEMORY_RANGE
}
