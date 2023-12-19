// serial driver.

cfg_if::cfg_if!(
    if #[cfg(any(feature = "qemu", feature = "pi4"))] {
        mod pl011;
        mod uart_pl011;
        pub use uart_pl011::*;
    } else if #[cfg(any(feature = "shyper", feature = "tx2", feature = "rk3588"))] {
        mod uart_ns16550;
        pub use uart_ns16550::*;
    }
);

pub fn init() {
    #[cfg(any(feature = "shyper", feature = "tx2", feature = "rk3588"))]
    uart_ns16550::init();
}
