pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

pub mod timer;
pub mod uart;
pub mod uart_qemu;
pub mod gic;

mod ns16550;
mod pl011;
