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

#[cfg(feature = "fat")]
pub mod blk;
#[cfg(feature = "net")]
pub mod net;
#[cfg(any(feature = "net", feature = "fat"))]
pub mod virtio;

#[cfg(feature = "pci")]
pub mod pci;

#[cfg(feature = "net")]
pub use net::get_network_driver;

pub mod error {
    #[cfg(any(feature = "net", feature = "fat"))]
    use crate::drivers::virtio::error::VirtioError;
    use core::fmt;

    #[derive(Debug)]
    pub enum DriverError {
        #[allow(dead_code)]
        CommonDevErr(u16),

        #[cfg(any(feature = "net", feature = "fat"))]
        InitVirtioDevFail(VirtioError),
    }

    #[cfg(any(feature = "net", feature = "fat"))]
    impl From<VirtioError> for DriverError {
        fn from(err: VirtioError) -> Self {
            DriverError::InitVirtioDevFail(err)
        }
    }

    impl fmt::Display for DriverError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                DriverError::CommonDevErr(err) => {
                    write!(f, "Common driver failed: {:?}", err)
                }
                #[cfg(any(feature = "net", feature = "fat"))]
                DriverError::InitVirtioDevFail(ref err) => {
                    write!(f, "Virtio driver failed: {:?}", err)
                }
            }
        }
    }
}

pub fn init_devices() {
    #[cfg(feature = "pci")]
    crate::drivers::pci::init();
    #[cfg(feature = "pci")]
    crate::drivers::pci::print_information();

    debug!("init virtio devices");
    #[cfg(any(feature = "net", feature = "fat"))]
    crate::drivers::virtio::init_drivers();
}
