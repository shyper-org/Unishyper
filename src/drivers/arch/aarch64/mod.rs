#[cfg(not(feature = "gicv3"))]
pub mod gic;
#[cfg(feature = "gicv3")]
pub mod gicv3;
pub mod psci;
pub mod timer;
pub mod uart;

#[cfg(not(feature = "gicv3"))]
pub use gic::{Interrupt, InterruptController};
#[cfg(feature = "gicv3")]
pub use gicv3::{Interrupt, InterruptController};
