use log::{Level, Metadata, Record};
use log::LevelFilter;

use crate::libs::synch::spinlock::SpinlockIrqSave;

struct SimpleLogger;

static LOCK: SpinlockIrqSave<()> = SpinlockIrqSave::new(());

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
        // metadata.level() <= Level::Debug
        // metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let _lock = LOCK.lock();
        if self.enabled(record.metadata()) {
            let ms = crate::libs::timer::current_ms();
            let s = ms / 1000;
            let ms = ms % 1000;
            print!("[{:04}.{:03}]", s, ms);

            match record.level() {
                Level::Error => print!("[E]"),
                Level::Warn => print!("[WARN]"),
                Level::Info => print!("[I]"),
                Level::Debug => print!("[D]"),
                Level::Trace => print!("[T]"),
            }
            if let Some(m) = record.module_path() {
                #[cfg(feature = "smp")]
                {
                    use crate::libs::traits::ArchTrait;
                    print!("core[{}][{}]", crate::arch::Arch::core_id(), m);
                }
                #[cfg(not(feature = "smp"))]
                print!("[{}]", m);
            }
            print!(" {}", record.args());
            println!();
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .expect("Panic on logger init failed");
    print_logo();
}

pub fn print_logo() {
    println!(concat!(
        "----------------------------------------------------\n\n",
        "     __  __      _      __                         \n",
        "    / / / /___  (_)____/ /_  __  ______  ___  _____\n",
        "   / / / / __ \\/ / ___/ __ \\/ / / / __ \\/ _ \\/ ___/\n",
        "  / /_/ / / / / (__  ) / / / /_/ / /_/ /  __/ /\n",
        "  \\____/_/ /_/_/____/_/ /_/\\__, / .___/\\___/_/\n",
        "                          /____/_/\n"
    ));
    println!(concat!(
        "----------------------------------------------------\n",
    ));
}
