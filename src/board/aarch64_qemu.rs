use core::ops::Range;

use crate::drivers::gic::INT_TIMER;
use crate::libs::traits::InterruptControllerTrait;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::Device;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::VirtioDevice;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0xc000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

pub const ELF_IMAGE_LOAD_ADDR: usize = 0x8000_0000;

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;

pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

cfg_if::cfg_if! {
    if #[cfg(all(feature = "axdriver", feature = "pci"))] {
        /// Base physical address of the PCIe ECAM space (should read from ACPI 'MCFG' table).
        pub const PCI_ECAM_BASE: usize = 0x40_1000_0000;
        /// End PCI bus number (`bus-range` property in device tree).
        pub const PCI_BUS_END: usize = 0xff;
        /// PCI device memory ranges (`ranges` property in device tree).
        pub const PCI_RANGES: &[(usize, usize)] = &[
            (0x3ef_f0000, 0x1_0000),          // PIO space
            (0x1000_0000, 0x2eff_0000),       // 32-bit MMIO space
            (0x80_0000_0000, 0x80_0000_0000), // 64-but MMIO space
        ];
    }
}

#[cfg(any(feature = "net", feature = "fat"))]
use alloc::{vec, vec::Vec};
#[cfg(any(feature = "net", feature = "fat"))]
#[allow(unused)]
pub fn devices() -> Vec<Device> {
    vec![
        #[cfg(feature = "fat")]
        Device::Virtio(VirtioDevice::new(
            "virtio_blk",
            0x0a00_0000..0x0a00_0200,
            0x10,
        )),
        #[cfg(feature = "net")]
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

#[no_mangle]
pub fn pmu_init() {
    // Init PMU.
    let pmcr = 1u64;
    let pmcntenset = 1u64 << 32;
    let pmuserenr = 1u64 << 2 | 1u64;
    unsafe {
        core::arch::asm!("msr pmcr_el0, {}", in(reg) pmcr);
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
    info!("starting to launch other cores...");
    extern "C" {
        fn KERNEL_ENTRY();
    }
    use crate::libs::traits::{ArchTrait, Address};
    let core_id = crate::arch::Arch::core_id();
    for i in 0..BOARD_CORE_NUMBER {
        if i != core_id {
            crate::drivers::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
        }
    }
}
