use super::pl011::Pl011Mmio;
use tock_registers::interfaces::{Readable, Writeable};

#[cfg(feature = "qemu")]
const PL011_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0x900_0000;

#[cfg(feature = "pi4")]
const PL011_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0xfe201000;

static PL011_MMIO: Pl011Mmio = Pl011Mmio::new(PL011_MMIO_BASE);

#[allow(unused)]
const UART_FR_RXFF: u32 = 1 << 4;
#[allow(unused)]
const UART_FR_TXFF: u32 = 1 << 5;

pub fn putc(c: u8) {
    if c == b'\n' {
        putc(b'\r');
    }
    let pl011 = &PL011_MMIO;
    loop {
        if pl011.Flag.get() & UART_FR_TXFF == 0 {
            break;
        }
    }
    pl011.Data.set(c as u32);
}

#[cfg(feature = "terminal")]
pub fn getc() -> Option<u8> {
    let pl011 = &PL011_MMIO;
    if pl011.Flag.get() & UART_FR_RXFF == 0 {
        Some((pl011.Data.get() & 0xff) as u8)
    } else {
        None
    }
}
