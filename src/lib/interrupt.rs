use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use spin::mutex::Mutex;

use crate::drivers::Interrupt;

// Todo: maybe try SpinlockIrqSave.
static IRQ_NAMES: Mutex<BTreeMap<u32, String>> = Mutex::new(BTreeMap::new());
static IRQ_HANDLERS: Mutex<BTreeMap<u32, fn()>> = Mutex::new(BTreeMap::new());

pub trait InterruptController {
    fn init(&self);

    fn enable(&self, int: Interrupt);
    fn disable(&self, int: Interrupt);

    fn fetch(&self) -> Option<Interrupt>;
    fn finish(&self, int: Interrupt);
}

#[no_mangle]
pub fn irq_install_handler(irq_number: u32, handler: fn(), name: &'static str) {
    debug!("[{}] Install handler for interrupt {}", name, irq_number);
    let mut irq_name_lock = IRQ_NAMES.lock();
    let mut irq_handler_lock = IRQ_HANDLERS.lock();

    irq_name_lock.insert(32 + irq_number, name.to_string());
    irq_handler_lock.insert(32 + irq_number, handler);
}

pub fn interrupt(int: Interrupt) {
    debug!("external {}", int);
}
