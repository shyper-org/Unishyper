#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/mod.rs"]
mod arch;

/// Pending:
/// Currently we use different serial driver implementations
/// for different architectures and platforms.
/// They need to be refactor in the future.
/// see arch/{target_arch}/uart for details.
pub use arch::*;
pub use arch::{Interrupt, INTERRUPT_CONTROLLER};

#[cfg(feature = "fat")]
pub mod blk;
#[cfg(feature = "tcp")]
pub mod net;
#[cfg(any(feature = "tcp", feature = "fat"))]
pub mod virtio;

pub mod error {
    #[cfg(any(feature = "tcp", feature = "fat"))]
    use crate::drivers::virtio::error::VirtioError;
    use core::fmt;

    #[derive(Debug)]
    pub enum DriverError {
        #[allow(dead_code)]
        CommonDevErr(u16),

        #[cfg(any(feature = "tcp", feature = "fat"))]
        InitVirtioDevFail(VirtioError),
    }

    #[cfg(any(feature = "tcp", feature = "fat"))]
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
                #[cfg(any(feature = "tcp", feature = "fat"))]
                DriverError::InitVirtioDevFail(ref err) => {
                    write!(f, "Virtio driver failed: {:?}", err)
                }
            }
        }
    }
}

pub fn init_devices() {
    info!("init virtio devices");
    #[cfg(any(feature = "tcp", feature = "fat"))]
    crate::drivers::virtio::init_drivers();
}
