use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

use crate::drivers::Interrupt;

// Todo: maybe try SpinlockIrqSave.
static IRQ_NAMES: BTreeMap<u32, String> = BTreeMap::new();
static IRQ_HANDLERS: BTreeMap<u32, fn()> = BTreeMap::new();

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<Interrupt>;
  fn finish(&self, int: Interrupt);
}



#[no_mangle]
pub fn irq_install_handler(irq_number: u32, handler: fn(), name: &'static str) {
	debug!("[{}] Install handler for interrupt {}",name, irq_number);
  IRQ_NAMES.insert(32 + irq_number, name.to_string());
  IRQ_HANDLERS.insert(32 + irq_number, handler);
}



pub fn interrupt(int: Interrupt) {
  debug!("external {}", int);
  
}