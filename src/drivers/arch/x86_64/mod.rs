pub mod timer;
pub mod uart;
pub mod rtc;
pub mod apic;
mod uart_16550;

pub use self::apic::{Interrupt, INTERRUPT_CONTROLLER};
