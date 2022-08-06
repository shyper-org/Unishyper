use core::fmt;
use core::fmt::Write;

use crate::lib::synch::spinlock::SpinlockIrqSave;

pub struct Writer;

static LOCK: SpinlockIrqSave<Writer> = SpinlockIrqSave::new(Writer);

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            crate::drivers::uart::putc(b);
        }
        Ok(())
    }
}

pub fn print_arg(args: fmt::Arguments) {
    let mut lock = LOCK.lock();
    lock.write_fmt(args).unwrap();
}
