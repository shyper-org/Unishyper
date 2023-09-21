use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

pub use crate::drivers::Interrupt;

use crate::libs::traits::InterruptControllerTrait;
use crate::libs::synch::spinlock::SpinlockIrqSave;

static IRQ_NAMES: SpinlockIrqSave<BTreeMap<u32, String>> = SpinlockIrqSave::new(BTreeMap::new());
static IRQ_HANDLERS: SpinlockIrqSave<BTreeMap<u32, fn()>> = SpinlockIrqSave::new(BTreeMap::new());

#[cfg(not(target_arch = "x86_64"))]
pub fn irq_install_handler(irq_number: u32, handler: fn(), name: &str) {
    info!(
        "[{}] Install handler for interrupt {} irq_num [32+{}]",
        name, irq_number, irq_number
    );
    let mut irq_name_lock = IRQ_NAMES.lock();
    let mut irq_handler_lock = IRQ_HANDLERS.lock();

    irq_name_lock.insert(32 + irq_number, name.to_string());
    irq_handler_lock.insert(32 + irq_number, handler);

    crate::drivers::InterruptController::enable(32 + irq_number as usize);
}

#[cfg(target_arch = "x86_64")]
pub fn irq_install_handler(irq_number: u32, handler: usize, name: &'static str) {
    info!(
        "[{}] Install handler for interrupt {} irq_num [32+{}]",
        name, irq_number, irq_number
    );
    let mut irq_name_lock = IRQ_NAMES.lock();

    irq_name_lock.insert(32 + irq_number, name.to_string());
    // irq_handler_lock.insert(32 + irq_number, handler);

    crate::arch::irq_install_handler(irq_number, handler);
    crate::drivers::InterruptController::enable(irq_number as usize);
    // crate::drivers::InterruptController::enable(irq_number as usize);
}

pub fn interrupt(int: Interrupt) {
    // debug!("external interrupt {:#x} => {:#x}", int, int - 32);
    let lock = IRQ_HANDLERS.lock();
    // During exception handling, nested interrupt is not permitted.
    if let Some(handler) = lock.get(&(int as u32)) {
        handler();
    } else {
        error!("interrupt {} is not registered!!!", int);
    }
}
