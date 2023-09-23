pub mod hsm;
pub mod plic;
mod sbi;
pub mod timer;

#[cfg_attr(not(feature = "k210"), path = "uart_ns16550.rs")]
#[cfg_attr(feature = "k210", path = "uart_k210.rs")]
pub mod uart;

pub use plic::{Interrupt, InterruptController};
