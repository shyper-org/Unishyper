pub mod apic;
pub mod rtc;
pub mod timer;
pub mod uart;
mod uart_16550;

pub use self::apic::{Interrupt, INTERRUPT_CONTROLLER};
