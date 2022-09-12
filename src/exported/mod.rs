pub use thread::*;
pub use crate::mm::*;
pub use crate::lib::synch::{spinlock, semaphore};
pub use crate::lib::thread::*;
pub use crate::lib::timer::*;

mod thread;

pub fn core_id() -> usize {
    use crate::lib::traits::ArchTrait;
    crate::arch::Arch::core_id()
}

#[cfg(feature = "tcp")]
pub mod net;
#[cfg(feature = "tcp")]
pub use net::*;

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "fs")]
pub mod fd;
#[cfg(feature = "fs")]
pub mod io;