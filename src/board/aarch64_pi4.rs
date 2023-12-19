use core::ops::Range;

use crate::drivers::gic::INT_TIMER;

use crate::libs::traits::InterruptControllerTrait;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x0000_0000..0xc000_0000; // Actually it starts from 0xfc000000.
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0xc000_0000..0x1_0000_0000;

pub const ELF_IMAGE_LOAD_ADDR: usize = 0x8000_0000;

pub const GICD_BASE: usize = 0xff841000;
pub const GICC_BASE: usize = 0xff842000;

pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

cfg_if::cfg_if! {
    if #[cfg(all(feature = "axdriver", feature = "pci"))] {
        // Base physical address of the PCIe ECAM space (should read from ACPI 'MCFG' table).
        pub const PCI_ECAM_BASE: usize = 0;
        /// End PCI bus number.
        pub const PCI_BUS_END: usize = 0;
        /// PCI device memory ranges.
        pub const PCI_RANGES: &[(usize, usize)] = &[];
    }
}

#[cfg(any(feature = "net", feature = "fat"))]
use {alloc::vec::Vec, alloc::vec, crate::libs::device::Device};
#[cfg(any(feature = "net", feature = "fat"))]
pub fn devices() -> Vec<Device> {
    vec![]
}

pub fn init() {
    crate::drivers::init_devices();
}

#[no_mangle]
pub fn pmu_init() {
    // Init PMU.
    let mut pmcr: u32;
    // Performance Monitors Count Enable Clear register.
    let pmcntenclr = u32::MAX as u64;
    let pmcntenset = 1u64 << 31;
    let pmuserenr = 1u64 << 2 | 1u64;
    unsafe {
        core::arch::asm!("msr pmcntenclr_el0, {}", in(reg) pmcntenclr);

        core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
        pmcr &= !(1u32 << 3);
        core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

        core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
        pmcr |= (1u32 << 1) | (1u32 << 2);
        core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

        core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
        pmcr |= 1;
        core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

        core::arch::asm!("msr pmcntenset_el0, {}", in(reg) pmcntenset);
        core::arch::asm!("msr pmuserenr_el0, {}", in(reg) pmuserenr);
    }
}

pub fn init_per_core() {
    // Init interrupt controller.
    use cortex_a::registers::*;
    use tock_registers::interfaces::Writeable;
    DAIF.write(DAIF::I::Masked);
    crate::drivers::InterruptController::init();
    crate::drivers::InterruptController::enable(INT_TIMER);
    crate::drivers::timer::init();

    // Init page table.
    crate::arch::page_table::install_page_table();

    pmu_init();
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    extern "C" {
        fn KERNEL_ENTRY();
    }
    use crate::libs::traits::{ArchTrait, Address};
    let core_id = crate::arch::Arch::core_id();
    for i in 0..BOARD_CORE_NUMBER {
        if i != core_id {
            crate::driver::psci::cpu_on(
                (id as u64) | (1 << 31),
                (KERNEL_ENTRY as usize).kva2pa() as u64,
                0,
            );
        }
    }
}
