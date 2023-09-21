pub mod gic;
pub mod psci;
pub mod timer;
pub mod uart;

pub use gic::{Interrupt, InterruptController};
