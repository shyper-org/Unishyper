use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

#[cfg(any(feature = "tcp", feature = "fat"))]
use crate::libs::device::Device;
#[cfg(any(feature = "tcp", feature = "fat"))]
use crate::libs::device::VirtioDevice;
use crate::libs::interrupt::InterruptController;
use crate::libs::traits::*;

pub const BOARD_CORE_NUMBER: usize = 1;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xf000_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub const ELF_IMAGE_LOAD_ADDR: usize = 0xc0000000;

pub const GLOBAL_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

pub fn init() {
    crate::drivers::uart::init();
    crate::drivers::init_devices();
}

pub fn init_per_core() {
    crate::drivers::timer::init();
    crate::arch::Arch::exception_init();
    crate::drivers::INTERRUPT_CONTROLLER.init();
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

#[cfg(any(feature = "tcp", feature = "fat"))]
use alloc::{vec, vec::Vec};
#[cfg(any(feature = "tcp", feature = "fat"))]
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
