use crate::lib::interrupt::InterruptController;
use crate::drivers::gic::INT_TIMER;

pub fn init(){
    use cortex_a::registers::*;
    use tock_registers::interfaces::Writeable;
    DAIF.write(DAIF::I::Masked);
    crate::drivers::INTERRUPT_CONTROLLER.init();
    crate::drivers::INTERRUPT_CONTROLLER.enable(INT_TIMER);
    crate::drivers::timer::init();
    let pmcr = 1u64;
    let pmcntenset = 1u64 << 32;
    let pmuserenr = 1u64 << 2 | 1u64;
    unsafe {
      core::arch::asm!("msr pmcr_el0, {}", in(reg) pmcr);
      core::arch::asm!("msr pmcntenset_el0, {}", in(reg) pmcntenset);
      core::arch::asm!("msr pmuserenr_el0, {}", in(reg) pmuserenr);
    }
  }