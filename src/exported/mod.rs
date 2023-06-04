// pub use crate::libs::synch::{spinlock, semaphore};
pub use crate::libs::thread::*;
pub use crate::libs::timer::*;

mod mm;
pub use mm::*;

#[cfg(feature = "std")]
mod abicalls;
#[cfg(feature = "std")]
pub use abicalls::*;

pub use crate::libs::thread::thread_exit as exit;

pub fn core_id() -> usize {
    use crate::libs::traits::ArchTrait;
    crate::arch::Arch::core_id()
}

#[cfg(feature = "tcp")]
pub mod net;
#[cfg(feature = "tcp")]
pub use net::*;

#[cfg(all(feature = "fs", feature = "fat"))]
pub mod fd;
#[cfg(all(feature = "fs", feature = "fat"))]
pub mod fs;
#[cfg(all(feature = "fs", feature = "fat"))]
pub mod io;
