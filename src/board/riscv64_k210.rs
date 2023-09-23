use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::Device;
#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::VirtioDevice;
use crate::libs::traits::*;

pub const BOARD_CORE_NUMBER: usize = 1;

/// MaixDock(M1) only has 8MiB 64bit on-chip SRAM.
/// Ref: https://wiki.sipeed.com/soft/maixpy/en/develop_kit_board/maix_dock.html
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0x8060_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub const ELF_IMAGE_LOAD_ADDR: usize = 0xdeadbeef;

pub const GLOBAL_HEAP_SIZE: usize = 4 * 1024 * 1024; // 16 MB

cfg_if::cfg_if! {
    if #[cfg(all(feature = "axdriver", feature = "pci"))] {
        /// Base physical address of the PCIe ECAM space (should read from ACPI 'MCFG' table).
        pub const PCI_ECAM_BASE: usize = 0x3000_0000;
        /// End PCI bus number (`bus-range` property in device tree).
        pub const PCI_BUS_END: usize = 0xff;
        /// PCI device memory ranges (`ranges` property in device tree).
        pub const PCI_RANGES: &[(usize, usize)] = &[
            (0x0300_0000, 0x1_0000),        // PIO space
            (0x4000_0000, 0x4000_0000),     // 32-bit MMIO space
            (0x4_0000_0000, 0x4_0000_0000), // 64-but MMIO space
        ];
    }
}

pub fn init() {
    crate::drivers::uart::init();
    crate::drivers::init_devices();
}

pub fn init_per_core() {
    crate::drivers::timer::init();
    crate::arch::Arch::exception_init();
    crate::drivers::InterruptController::init();
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    HART_SPIN.store(true, Ordering::Relaxed);
}

static HART_SPIN: AtomicBool = AtomicBool::new(false);
static HART_BOOT: Mutex<Option<usize>> = Mutex::new(None);

#[no_mangle]
pub unsafe extern "C" fn print_arg(arg0: usize, arg1: usize, arg2: usize) {
    println!("enter RISCV print_arg, arg0 {:#x} arg1 {:#x} arg2 {:#x}", arg0, arg1, arg2);
}

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
    extern "C" {
        fn KERNEL_ENTRY();
    }

    let mut hart_boot = HART_BOOT.lock();
    if hart_boot.is_none() {
        *hart_boot = Some(core_id);
        drop(hart_boot);
        for i in 0..BOARD_CORE_NUMBER {
            if i != core_id {
                let _ = crate::drivers::hsm::hart_start(i, (KERNEL_ENTRY as usize).kva2pa(), 0);
            }
        }
    } else {
        drop(hart_boot);
    }

    if core_id == 0 {
        crate::loader_main(core_id);
    }
    while !HART_SPIN.load(Ordering::Relaxed) {}
    crate::loader_main(core_id);
}

#[cfg(any(feature = "net", feature = "fat"))]
use alloc::{vec, vec::Vec};
#[cfg(any(feature = "net", feature = "fat"))]
pub fn devices() -> Vec<Device> {
    vec![
        Device::new("GPIOHS", vec![0x3800_1000..0x3800_2000], vec![]),
        Device::new("SPI0", vec![0x5200_0000..0x5200_1000], vec![]),
        Device::new("DMAC", vec![0x5000_0000..0x5000_1000], vec![]),
        Device::new("SYSCTL", vec![0x5044_0000..0x5044_1000], vec![]),
        Device::new("FPIOA", vec![0x502B_0000..0x502B_1000], vec![]),
    ]
}
