// use core::ops::Range;

use crate::libs::interrupt::InterruptController;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub const ELF_IMAGE_LOAD_ADDR: usize = 0xdeafbeef;

pub fn init() {
    crate::arch::init_idt();
    crate::drivers::init_devices();
}

pub fn init_per_core() {
    crate::drivers::INTERRUPT_CONTROLLER.init();
    crate::drivers::timer::init();
    crate::arch::irq::disable();
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    warn!("Unimplented!!! starting to launch other cores...");
}
