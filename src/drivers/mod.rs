pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

pub mod gic;
pub mod psci;
mod smc;
pub mod timer;
pub mod uart;

#[cfg(feature = "fs")]
pub mod blk;
#[cfg(feature = "tcp")]
pub mod net;
#[cfg(any(feature = "tcp", feature = "fs"))]
pub mod virtio;

pub mod error {
    #[cfg(any(feature = "tcp", feature = "fs"))]
    use crate::drivers::virtio::error::VirtioError;
    use core::fmt;

    #[derive(Debug)]
    pub enum DriverError {
        #[allow(dead_code)]
        CommonDevErr(u16),

        #[cfg(any(feature = "tcp", feature = "fs"))]
        InitVirtioDevFail(VirtioError),
    }

    #[cfg(any(feature = "tcp", feature = "fs"))]
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
                #[cfg(any(feature = "tcp", feature = "fs"))]
                DriverError::InitVirtioDevFail(ref err) => {
                    write!(f, "Virtio driver failed: {:?}", err)
                }
            }
        }
    }
}

pub fn init_devices() {
    info!("init virtio devices");
    #[cfg(any(feature = "tcp", feature = "fs"))]
    crate::drivers::virtio::init_drivers();
}
