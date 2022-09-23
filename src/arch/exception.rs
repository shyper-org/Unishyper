use core::mem::size_of;
use cortex_a::registers::{ESR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::libs::traits::ArchTrait;
use crate::libs::traits::ContextFrameTrait;

use crate::arch::ContextFrame;

core::arch::global_asm!(include_str!("exception.S"));

#[no_mangle]
unsafe extern "C" fn current_el_sp0_synchronous(ctx: *mut ContextFrame) {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    panic!("current_el_sp0_synchronous EC {:#X} \n{}", ec, ctx.read());
    // loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el_sp0_irq(ctx: *mut ContextFrame) {
    // trace!("current_el_sp0_irq \n{}", ctx.read());
    use crate::libs::interrupt::*;
    let core = crate::libs::cpu::cpu();
    core.set_context(ctx);
    use crate::drivers::{gic::INT_TIMER, INTERRUPT_CONTROLLER};
    let irq = INTERRUPT_CONTROLLER.fetch();
    match irq {
        Some(INT_TIMER) => {
            crate::libs::timer::interrupt();
        }
        Some(i) => {
            if i >= 32 {
                crate::libs::interrupt::interrupt(i);
            } else {
                panic!("GIC unhandled SGI PPI")
            }
        }
        None => {
            panic!("GIC unknown irq")
        }
    }
    if irq.is_some() {
        INTERRUPT_CONTROLLER.finish(irq.unwrap());
    }
    core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_synchronous(ctx: *mut ContextFrame) {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    error!("current_el_spx_synchronous EC {:#X} \n{}", ec, ctx.read());
    let ctx_mut = ctx.as_mut().unwrap();
    ctx_mut.set_stack_pointer(ctx as usize + size_of::<ContextFrame>());
    // let page_fault = ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortCurrentEL)
    //     | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortCurrentEL);
    //   crate::libs::exception::handle_kernel(ctx.as_ref().unwrap(), page_fault);
    panic!("current_el_spx_synchronous EC {:#X} \n{}", ec, ctx.read());
    // loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_irq(ctx: *mut ContextFrame) {
    trace!("current_el_spx_irq");
    current_el_sp0_irq(ctx);
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_serror(ctx: *mut ContextFrame) {
    panic!("current_el_spx_serror\n{}", ctx.read());
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(ctx: *mut ContextFrame) {
    let core_id = crate::arch::Arch::core_id();
    info!(
        "core {} lower_aarch64_synchronous\n {}",
        core_id,
        ctx.read()
    );
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(ctx: *mut ContextFrame) {
    let core = crate::libs::cpu::cpu();
    let core_id = crate::arch::Arch::core_id();

    core.set_context(ctx);
    info!(
        "core {} lower_aarch64_irq EL{} \n {}",
        core_id,
        crate::arch::Arch::curent_privilege(),
        ctx.read()
    );
    core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(ctx: *mut ContextFrame) {
    let core = crate::libs::cpu::cpu();

    let core_id = crate::arch::Arch::core_id();
    core.set_context(ctx);
    info!("core {} lower_aarch64_serror\n {}", core_id, ctx.read());
    core.clear_context();
}

pub fn init() {
    extern "C" {
        fn vectors();
    }
    unsafe {
        let addr: u64 = vectors as usize as u64;
        VBAR_EL1.set(addr);
        use cortex_a::asm::barrier::*;
        isb(SY);
    }
}
