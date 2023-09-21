#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/mod.rs"]
mod arch;

/// Pending:
/// Currently we use different serial driver implementations
/// for different architectures and platforms.
/// They need to be refactor in the future.
/// see arch/{target_arch}/uart for details.
pub use arch::*;
pub use arch::{Interrupt, InterruptController};

#[cfg(any(
    feature = "tx2",
    feature = "shyper",
    all(target_arch = "riscv64", feature = "qemu")
))]
mod ns16550;

#[cfg(any(feature = "net", feature = "fat"))]
pub mod axdriver;

use crate::libs::synch::spinlock::SpinlockIrqSave;
use alloc::vec::Vec;

pub enum AxDriver {
    #[cfg(any(feature = "net", feature = "fat"))]
    VirtioNet(SpinlockIrqSave<axdriver::AxNetDevice>),
}

impl AxDriver {
    #[cfg(any(feature = "net", feature = "fat"))]
    fn get_network_driver(&self) -> Option<&SpinlockIrqSave<axdriver::AxNetDevice>> {
        #[allow(unreachable_patterns)]
        match self {
            Self::VirtioNet(drv) => Some(drv),
            _ => None,
        }
    }
}

static mut AX_DRIVERS: Vec<AxDriver> = Vec::new();

fn register_driver(drv: AxDriver) {
    unsafe {
        AX_DRIVERS.push(drv);
    }
}

#[cfg(any(feature = "net", feature = "fat"))]
pub fn get_network_driver() -> Option<&'static SpinlockIrqSave<axdriver::AxNetDevice>> {
    unsafe { AX_DRIVERS.iter().find_map(|drv| drv.get_network_driver()) }
}

pub fn init_devices() {
    // #[cfg(feature = "pci")]
    // crate::drivers::pci::init();
    // #[cfg(feature = "pci")]
    // crate::drivers::pci::print_information();

    debug!("init virtio devices");

    // crate::drivers::virtio::init_drivers();
    let mut all_devices = axdriver::init_drivers();

    // Register virtio_net driver.
    #[cfg(any(feature = "net", feature = "fat"))]
    register_driver(AxDriver::VirtioNet(SpinlockIrqSave::new(
        all_devices.net.take_one().unwrap(),
    )));
}
