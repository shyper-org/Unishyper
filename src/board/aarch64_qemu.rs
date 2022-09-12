use alloc::vec::Vec;

use crate::drivers::gic::INT_TIMER;
use crate::lib::interrupt::InterruptController;
use crate::lib::device::Device;

#[cfg(any(feature = "tcp", feature = "fs"))]
use crate::lib::device::VirtioDevice;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;

#[allow(unused)]
pub fn devices() -> Vec<Device> {
    vec![
        #[cfg(feature = "fs")]
        Device::Virtio(VirtioDevice::new(
            "virtio_blk",
            0x0a00_0000..0x0a00_0200,
            0x10,
        )),
        #[cfg(feature = "tcp")]
        Device::Virtio(VirtioDevice::new(
            "virtio_net",
            0x0a00_3e00..0x0a00_4000,
            0x2f,
        )),
    ]
}

pub fn init() {
    crate::drivers::init_devices();
}

pub fn init_per_core() {
    use cortex_a::registers::*;
    use tock_registers::interfaces::Writeable;
    DAIF.write(DAIF::I::Masked);
    crate::drivers::INTERRUPT_CONTROLLER.init();
    crate::drivers::INTERRUPT_CONTROLLER.enable(INT_TIMER);
    crate::drivers::timer::init();
    // DAIF.write(DAIF::I::Unmasked);

    let pmcr = 1u64;
    let pmcntenset = 1u64 << 32;
    let pmuserenr = 1u64 << 2 | 1u64;
    unsafe {
        core::arch::asm!("msr pmcr_el0, {}", in(reg) pmcr);
        core::arch::asm!("msr pmcntenset_el0, {}", in(reg) pmcntenset);
        core::arch::asm!("msr pmuserenr_el0, {}", in(reg) pmuserenr);
    }
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    info!("starting to launch other cores...");
    extern "C" {
        fn KERNEL_ENTRY();
    }
    use crate::lib::traits::{ArchTrait, Address};
    let core_id = crate::arch::Arch::core_id();
    for i in 0..BOARD_CORE_NUMBER {
        if i != core_id {
            crate::drivers::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
        }
    }
}
