use core::fmt;
use core::fmt::Write;

use crate::libs::synch::spinlock::SpinlockIrqSave;

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


#[cfg(feature = "terminal")]
pub fn getchar() -> u8 {
    loop {
        let char = crate::libs::terminal::get_buffer_char();
        match char {
            0 => crate::libs::thread::thread_yield(),
            8 | 127 => break 127, // backspace
            b'\r' | 32..=126 => {
                // carriage return or visible
                let c = char as u8;
                print!("{}", c as char);
                break c;
            }
            _ => continue,
        }
    }
}

#[cfg(feature = "terminal")]
use alloc::string::String;
#[cfg(feature = "terminal")]
pub fn getline() -> String {
    use alloc::vec::Vec;
    let mut v = Vec::new();
    loop {
        let c = getchar();
        if c == b'\r' {
            break;
        }
        if c == 127 {
            if !v.is_empty() {
                crate::drivers::uart::putc(c);
            }
            v.pop();
            continue;
        }
        v.push(c);
    }
    String::from_utf8(v).expect("getline failed!")
}
