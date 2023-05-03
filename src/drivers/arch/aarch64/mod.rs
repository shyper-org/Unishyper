pub mod gic;
mod psci;
pub mod timer;
pub mod uart;

pub use gic::{Interrupt, INTERRUPT_CONTROLLER};