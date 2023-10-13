// serial driver.
// #[cfg(any(feature = "shyper", feature = "tx2"))]
// mod ns16550;
#[cfg(feature = "qemu")]
mod pl011;

// uart driver.
#[cfg(feature = "qemu")]
mod uart_qemu;
#[cfg(any(feature = "shyper", feature = "tx2", feature = "rk3588"))]
mod uart_shyper;

// export api.
#[cfg(feature = "qemu")]
pub use uart_qemu::*;
#[cfg(any(feature = "shyper", feature = "tx2", feature = "rk3588"))]
pub use uart_shyper::*;

pub fn init() {
    #[cfg(any(feature = "shyper", feature = "tx2", feature = "rk3588"))]
    uart_shyper::init();
}
