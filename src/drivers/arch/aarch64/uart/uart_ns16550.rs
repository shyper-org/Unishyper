use crate::drivers::ns16550::*;
use tock_registers::interfaces::{Readable, Writeable};

// The real mmio addressed of serial on Nvidia tx2 are 0xc280000 and 0x3100000.
#[cfg(feature = "tx2")]
const NS16550_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0x3100000;

#[cfg(feature = "rk3588")]
const NS16550_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0xfeb50000;

// The ipa provided by the hypervisor is 0x9000000.
#[cfg(feature = "shyper")]
const NS16550_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0xc280000;

static NS16550_MMIO: Ns16550Mmio32 = Ns16550Mmio32::new(NS16550_MMIO_BASE);

pub fn init() {
    let uart = &NS16550_MMIO;
    uart.ISR_FCR.write(ISR_FCR::EN_FIFO::Mode16550);
}

fn send(c: u8) {
    let uart = &NS16550_MMIO;
    while uart.LSR.get() & 0x20 == 0 {
        // Wait until it is possible to write data.
    }
    uart.RHR_THR_DLL.set(c);
}

pub fn putc(c: u8) {
    if c == b'\0' {
        return;
    }
    if c == b'\n' {
        send(b'\r');
    }
    send(c);
}

#[cfg(feature = "terminal")]
pub fn getc() -> Option<u8> {
    let uart = &NS16550_MMIO;
    if uart.LSR.get() & 0x1 != 0 {
        Some(uart.RHR_THR_DLL.get() as u8)
    } else {
        None
    }
}
