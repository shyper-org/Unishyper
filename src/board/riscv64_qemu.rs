use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::Device;
#[cfg(any(feature = "net", feature = "fat"))]
use crate::libs::device::VirtioDevice;
use crate::libs::traits::*;

pub const BOARD_CORE_NUMBER: usize = 1;

#[allow(dead_code)]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xf000_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub const ELF_IMAGE_LOAD_ADDR: usize = 0xc000_0000;

pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

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
    crate::drivers::InterruptController::init();
}

#[cfg(feature = "smp")]
pub fn launch_other_cores() {
    HART_SPIN.store(true, Ordering::Relaxed);
}

static HART_SPIN: AtomicBool = AtomicBool::new(false);
static HART_BOOT: Mutex<Option<usize>> = Mutex::new(None);

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
        Device::Virtio(VirtioDevice::new(
            "virtio_blk",
            0x10001000..0x10002000,
            0x10,
        )),
        // Device::new("rtc", vec![0x101000..0x102000], vec![]),
    ]
}
