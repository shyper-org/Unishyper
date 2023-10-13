#[cfg_attr(not(feature = "gicv3"), path = "gic.rs")]
#[cfg_attr(feature = "gicv3", path = "gicv3.rs")]
pub mod gic;
pub mod psci;
pub mod timer;
pub mod uart;

pub use gic::{Interrupt, InterruptController};
