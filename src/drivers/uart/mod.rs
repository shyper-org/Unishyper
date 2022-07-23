// serial driver.
#[cfg(feature = "qemu")]
mod pl011;
#[cfg(feature = "shyper")]
mod ns16550;

// uart driver.
#[cfg(feature = "qemu")]
mod uart_qemu;
#[cfg(feature = "shyper")]
mod uart_shyper;

// export api.
#[cfg(feature = "qemu")]
pub use uart_qemu::*;
#[cfg(feature = "shyper")]
pub use uart_shyper::*;

