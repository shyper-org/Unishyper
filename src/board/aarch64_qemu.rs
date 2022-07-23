use crate::drivers::gic::INT_TIMER;
use crate::lib::interrupt::InterruptController;
use crate::lib::traits::{ArchTrait, Address};

pub const BOARD_CORE_NUMBER: usize = 1;

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;

pub const VIRTIO_MMIO_START: usize = 0xFFFF_FF80_0000_0000 | 0x0a00_3e00;
pub const VIRTIO_MMIO_END: usize = 0xFFFF_FF80_0000_0000 | 0x0a00_4000;
pub const VIRTIO_NET_IRQ_NUMBER: u32 = 0x2f;

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

pub fn launch_other_cores() {
    extern "C" {
        fn KERNEL_ENTRY();
    }
    let core_id = crate::arch::Arch::core_id();
    for i in 0..BOARD_CORE_NUMBER {
        if i != core_id {
            crate::drivers::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
        }
    }
}
