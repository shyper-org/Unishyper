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

pub fn kernel_range() -> Range<usize> {
    let normal_range = crate::arch::BOARD_NORMAL_MEMORY_RANGE;
    normal_range.start..kernel_end_address().value()
}

pub fn heap_range() -> Range<usize> {
    kernel_end_address().value()..crate::arch::ELF_IMAGE_LOAD_ADDR
}

pub fn elf_range() -> Range<usize> {
    use crate::arch::{ELF_IMAGE_LOAD_ADDR, ELF_SIZE};
    ELF_IMAGE_LOAD_ADDR..ELF_IMAGE_LOAD_ADDR + ELF_SIZE
}

pub fn paged_range() -> Range<usize> {
    let normal_range = crate::arch::BOARD_NORMAL_MEMORY_RANGE;
    elf_range().end..normal_range.end
}
