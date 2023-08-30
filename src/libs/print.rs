use core::fmt;

#[cfg(feature = "serial")]
use core::fmt::Write;

#[cfg(feature = "serial")]
use crate::libs::synch::spinlock::SpinlockIrqSave;

#[cfg(feature = "serial")]
pub struct Writer;

#[cfg(feature = "serial")]
static LOCK: SpinlockIrqSave<Writer> = SpinlockIrqSave::new(Writer);

#[cfg(feature = "serial")]
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            crate::drivers::uart::putc(b);
        }
        Ok(())
    }
}

pub fn print_arg(_args: fmt::Arguments) {
    #[cfg(feature = "serial")]
    {
        let mut lock = LOCK.lock();
        lock.write_fmt(_args).unwrap();
    }
}

pub fn print_byte(buf: &[u8]) {
    #[cfg(feature = "serial")]
    {
        let _lock = LOCK.lock();
        for b in buf {
            crate::drivers::uart::putc(*b);
        }
    }
}

#[cfg(feature = "terminal")]
pub fn getchar() -> u8 {
    loop {
        let char = crate::libs::terminal::get_buffer_char();
        match char {
            0 => crate::libs::thread::thread_yield(),
            8 | 127 => break 127, // backspace
            b'\n' => {
                let c = char as u8;
                break c;
            }
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
    let mut v = alloc::vec::Vec::new();
    loop {
        let c = getchar();
        if c == b'\r' || c == b'\n' {
            break;
        }
        if c == 127 {
            if !v.is_empty() {
                crate::drivers::uart::putc(8);
                crate::drivers::uart::putc(b' ');
                crate::drivers::uart::putc(8);
            }
            v.pop();
            continue;
        }
        v.push(c);
    }
    String::from_utf8(v).expect("getline failed!")
}
