use crate::lib::interrupt::InterruptController;
use crate::driver::gic::INT_TIMER;

pub fn init(){
    use cortex_a::registers::*;
    use tock_registers::interfaces::Writeable;
    DAIF.write(DAIF::I::Masked);
    crate::driver::INTERRUPT_CONTROLLER.init();
    crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
    crate::driver::timer::init();
    let pmcr = 1u64;
    let pmcntenset = 1u64 << 32;
    let pmuserenr = 1u64 << 2 | 1u64;
    unsafe {
      core::arch::asm!("msr pmcr_el0, {}", in(reg) pmcr);
      core::arch::asm!("msr pmcntenset_el0, {}", in(reg) pmcntenset);
      core::arch::asm!("msr pmuserenr_el0, {}", in(reg) pmuserenr);
    }
  }