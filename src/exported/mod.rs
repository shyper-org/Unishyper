// pub use crate::libs::synch::{spinlock, semaphore};

// Todo: remove these exports.
pub use crate::libs::thread::*;
pub use crate::libs::timer::*;

// mod mm;
// pub use mm::*;

#[cfg(feature = "std")]
mod abicalls;
#[cfg(feature = "std")]
pub use abicalls::*;

// #[cfg(not(feature = "std"))]
pub mod shyperstd;

pub use crate::libs::thread::thread_exit as exit;

// pub fn core_id() -> usize {
//     use crate::libs::traits::ArchTrait;
//     crate::arch::Arch::core_id()
// }

// pub mod io;
// pub mod thread;

// #[cfg(feature = "tcp")]
// pub mod net;
// #[cfg(feature = "tcp")]
// pub use net::*;

// #[cfg(feature = "fs")]
// pub mod fd;
// #[cfg(feature = "fs")]
// pub mod fs;
