use super::uart_16550::SerialPort;

const SERIAL_IO_PORT: u16 = 0x3F8;

static mut COM1: SerialPort = unsafe { SerialPort::new(SERIAL_IO_PORT) };

pub fn init() {
    unsafe { COM1.init() };
}

pub fn putc(byte: u8) {
    unsafe { COM1.send(byte) };
}

#[cfg(feature = "terminal")]
pub fn getc() -> Option<u8> {
    unsafe { COM1.receive() }
}
