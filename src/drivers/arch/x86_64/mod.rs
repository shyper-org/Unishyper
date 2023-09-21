pub mod apic;
pub mod rtc;
pub mod timer;
pub mod uart;
mod uart_16550_port;

pub use self::apic::{Interrupt, InterruptController};
