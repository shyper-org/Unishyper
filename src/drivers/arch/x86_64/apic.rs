use apic::{LocalApic, XApic, LAPIC_ADDR};
use crate::libs::traits::Address;

use crate::libs::interrupt::InterruptController;

pub struct Apic;

#[allow(unused)]
impl InterruptController for Apic {
    fn init(&self) {
        let mut lapic = unsafe { XApic::new(LAPIC_ADDR.pa2kva()) };
        lapic.cpu_init();
        crate::util::barrier();

        debug!("apic init ok");
    }

    fn enable(&self, int: Interrupt) {}
    fn disable(&self, int: Interrupt) {}

    fn fetch(&self) -> Option<Interrupt> {
        unimplemented!();
    }
    fn finish(&self, int: Interrupt) {
        let mut lapic = unsafe { XApic::new(LAPIC_ADDR.pa2kva()) };
        lapic.eoi();
    }
}

pub const IRQ_MIN: usize = 0x20;
pub const INT_TIMER: Interrupt = IRQ_MIN + 0; // virtual timer

pub type Interrupt = usize;
pub static INTERRUPT_CONTROLLER: Apic = Apic {};
