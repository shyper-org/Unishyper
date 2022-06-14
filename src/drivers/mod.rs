pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

pub mod timer;
pub mod uart;
pub mod gic;

pub mod net;
pub mod virtio;

pub mod error {
	use crate::drivers::virtio::error::VirtioError;
	use core::fmt;

	#[derive(Debug)]
	pub enum DriverError {
		InitVirtioDevFail(VirtioError),
	}

	impl From<VirtioError> for DriverError {
		fn from(err: VirtioError) -> Self {
			DriverError::InitVirtioDevFail(err)
		}
	}

	impl fmt::Display for DriverError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match *self {
				DriverError::InitVirtioDevFail(ref err) => {
					write!(f, "Virtio driver failed: {:?}", err)
				}
			}
		}
	}
}
pub fn init_drivers() {
	
}