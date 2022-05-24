use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use spin::Mutex;

struct SimpleLogger;

static LOCK: Mutex<()> = Mutex::new(());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::print::print_arg(format_args_nl!($($arg)*));
    })
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    error!("rust_eh_personality called");
    loop {}
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let lock = LOCK.lock();
        if self.enabled(record.metadata()) {
            let ms = crate::lib::timer::current_ms();
            let s = ms / 1000;
            let ms = ms % 1000;
            print!("[{:04}.{:03}]", s, ms);

            match record.level() {
                Level::Error => print!("[E]"),
                Level::Warn => print!("[W]"),
                Level::Info => print!("[I]"),
                Level::Debug => print!("[D]"),
                Level::Trace => print!("[T]"),
            }
            if let Some(m) = record.module_path() {
                print!("[{}]", m);
            }
            print!(" {}", record.args());
            println!();
        }
        drop(lock);
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}
