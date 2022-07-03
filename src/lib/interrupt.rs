use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

use crate::drivers::Interrupt;
use crate::lib::synch::spinlock::SpinlockIrqSave;

// Todo: maybe try SpinlockIrqSave.
static IRQ_NAMES: SpinlockIrqSave<BTreeMap<u32, String>> =SpinlockIrqSave::new(BTreeMap::new());
static IRQ_HANDLERS: SpinlockIrqSave<BTreeMap<u32, fn()>> = SpinlockIrqSave::new(BTreeMap::new());

pub trait InterruptController {
    fn init(&self);

    fn enable(&self, int: Interrupt);
    fn disable(&self, int: Interrupt);

    fn fetch(&self) -> Option<Interrupt>;
    fn finish(&self, int: Interrupt);
}

#[no_mangle]
pub fn irq_install_handler(irq_number: u32, handler: fn(), name: &'static str) {
    debug!("[{}] Install handler for interrupt {} irq_num [32+{}]", name, irq_number, irq_number);
    let mut irq_name_lock = IRQ_NAMES.lock();
    let mut irq_handler_lock = IRQ_HANDLERS.lock();

    irq_name_lock.insert(32 + irq_number, name.to_string());
    irq_handler_lock.insert(32 + irq_number, handler);

    crate::drivers::INTERRUPT_CONTROLLER.enable(32 + irq_number as usize);
}

pub fn interrupt(int: Interrupt) {
    debug!("external interrupt {}", int);
    let lock = IRQ_HANDLERS.lock();
    // During exception handling, nested interrupt is not permitted.
    if let Some(handler) = lock.get(&(int as u32)) {
        handler();
    } else {
        error!("interrupt {} is not registered!!!", int);
    }
}
