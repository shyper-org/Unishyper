use core::fmt;
use core::fmt::Write;

use spin::Mutex;

pub struct Writer;

static WRITER: Mutex<Writer> = Mutex::new(Writer);

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            crate::driver::uart_qemu::putc(b);
        }
        Ok(())
    }
}

pub fn print_arg(args: fmt::Arguments) {
    let mut lock = WRITER.lock();
    lock.write_fmt(args).unwrap();
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
    loop {}
}
