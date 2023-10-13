use core::ops::Range;

#[cfg(not(feature = "gicv3"))]
use crate::drivers::gic::INT_TIMER;
#[cfg(feature = "gicv3")]
use crate::drivers::gicv3::INT_TIMER;

use crate::libs::traits::InterruptControllerTrait;

#[cfg(not(feature = "smp"))]
pub const BOARD_CORE_NUMBER: usize = 1;

#[cfg(feature = "smp")]
pub const BOARD_CORE_NUMBER: usize = 2;

// On Nvidia tx2The real mmio addressed of gicd is 0x3881000, gicc is 0x3882000.

#[cfg(feature = "tx2")]
pub const GICD_BASE: usize = 0x3881000;
// The ipa of gicd provided by the hypervisor as a emulated device is 0x8000000.
#[cfg(feature = "rk3588")]
pub const GICD_BASE: usize = 0xfe600000;
#[cfg(feature = "shyper")]
pub const GICD_BASE: usize = 0x8000000;

#[cfg(feature = "tx2")]
pub const GICC_BASE: usize = 0x3882000;
#[cfg(feature = "rk3588")]
pub const GICC_BASE: usize = 0xfe610000;
// The ipa of gicc provided by the hypervisor as a passthrough device is 0x8010000.
#[cfg(feature = "shyper")]
pub const GICC_BASE: usize = 0x8010000;

#[cfg(feature = "tx2")]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xf000_0000;
#[cfg(feature = "tx2")]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;
#[cfg(feature = "tx2")]
pub const ELF_IMAGE_LOAD_ADDR: usize = 0xc000_0000;

#[cfg(feature = "rk3588")]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x0000_0000..0xc000_0000;
#[cfg(feature = "rk3588")]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0xc000_0000..0x1_0000_0000;
#[cfg(feature = "rk3588")]
pub const ELF_IMAGE_LOAD_ADDR: usize = 0x8000_0000;
// Todo: redesign memory range in shyper.
//       When running on hypervisor, unikernel should not take up so much memory region.
#[cfg(feature = "shyper")]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
#[cfg(feature = "shyper")]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;
#[cfg(feature = "shyper")]
pub const ELF_IMAGE_LOAD_ADDR: usize = 0x6000_0000;

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
    use crate::libs::device::VirtioDevice;
    vec![
        #[cfg(feature = "fat")]
        Device::Virtio(VirtioDevice::new(
            "virtio_blk",
            0x0a00_0000..0x0a00_0200,
            0x10,
        )),
        #[cfg(feature = "net")]
        Device::Virtio(VirtioDevice::new("virtio_net", 0xa001000..0xa002000, 0x11)),
    ]
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
            #[cfg(feature = "shyper")]
            crate::drivers::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
            #[cfg(feature = "tx2")]
            crate::driver::psci::cpu_on(
                (i as u64) | 0x80000100,
                (KERNEL_ENTRY as usize).kva2pa() as u64,
                0,
            );
            #[cfg(feature = "rk3588")]
            crate::driver::psci::cpu_on(
                ((i as u64) << 8) | 0x81000000,
                (KERNEL_ENTRY as usize).kva2pa() as u64,
                0,
            );
        }
    }
}
