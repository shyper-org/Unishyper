pub use crate::libs::synch::{spinlock, semaphore};
pub use crate::libs::thread::*;
pub use crate::libs::timer::*;

mod mm;
pub use mm::*;

pub use crate::libs::thread::thread_exit as exit;

pub fn core_id() -> usize {
    use crate::libs::traits::ArchTrait;
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