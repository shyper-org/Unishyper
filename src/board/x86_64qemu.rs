// use core::ops::Range;

use crate::libs::interrupt::InterruptController;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

// pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0xc000_0000;
// pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

// pub const ELF_IMAGE_LOAD_ADDR: usize = 0x8000_0000;
// pub const ELF_SIZE: usize = 0xa0_0000;

// pub const GICD_BASE: usize = 0x08000000;
// pub const GICC_BASE: usize = 0x08010000;

pub const KERNEL_HEAP_SIZE: usize = 8 * 1024 * 1024; // 8 MB


pub fn init() {
    crate::drivers::init_devices();
}

pub fn init_per_core() {
    crate::drivers::INTERRUPT_CONTROLLER.init();
    crate::drivers::timer::init();
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    warn!("Unimplented!!! starting to launch other cores...");
}
