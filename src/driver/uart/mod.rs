// serial driver.
#[cfg(feature = "qemu")]
mod pl011;
#[cfg(feature = "tx2")]
mod ns16550;

// uart driver.
#[cfg(feature = "qemu")]
mod uart_qemu;
#[cfg(feature = "tx2")]
mod uart_tx2;

// export api.
#[cfg(feature = "qemu")]
pub use uart_qemu::*;
#[cfg(feature = "tx2")]
pub use uart_tx2::*;

