use cortex_a::registers::{ESR_EL1, VBAR_EL1, TPIDRRO_EL0, DAIF};
use tock_registers::interfaces::{Readable, Writeable};

use crate::drivers::InterruptController;
use crate::libs::traits::ArchTrait;
use crate::libs::traits::InterruptControllerTrait;

use super::ContextFrame;

core::arch::global_asm!(include_str!("start.S"));

core::arch::global_asm!(
    include_str!("exception.S"),
    size_of_context_frame = const core::mem::size_of::<ContextFrame>()
);

#[no_mangle]
unsafe extern "C" fn current_el_spx_synchronous(ctx: *mut ContextFrame) {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    let tid = TPIDRRO_EL0.get();
    panic!(
        "current_el_spx_synchronous on Thread {}\nEC {:#X} ESR_EL1 {:#x}\n{}",
        tid,
        ec,
        ESR_EL1.get(),
        ctx.read()
    );
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_irq(ctx: *mut ContextFrame) {
    let irq = InterruptController::fetch();
    // debug!(
    //     "current_el_spx_irq, thread [{}], el{}, irq {}, daif: {:x}\n ctx on sp {:p}\n",
    //     TPIDRRO_EL0.get(),
    //     crate::arch::Arch::curent_privilege(),
    //     irq.unwrap(),
    //     DAIF.get(),
    //     ctx
    // );
    // println!("{}", ctx.read());

    // Store current context's pointer on current core struct.
    // Note: ctx is just a pointer to current core stack.
    let core = crate::libs::cpu::cpu();
    core.set_current_sp(ctx as usize);

    use crate::drivers::gic::INT_TIMER;
    match irq {
        Some(INT_TIMER) => {
            crate::libs::timer::interrupt();
            InterruptController::finish(INT_TIMER);
            // Give up CPU actively.
            crate::libs::thread::thread_yield();
        }
        Some(i) => {
            if i >= 32 {
                crate::libs::interrupt::interrupt(i);
                InterruptController::finish(irq.unwrap());
            } else {
                warn!(
                    "current_el_spx_irq, thread [{}], el{}, irq {}, daif: {:x}\n ctx on sp {:p}\n",
                    TPIDRRO_EL0.get(),
                    crate::arch::Arch::curent_privilege(),
                    irq.unwrap(),
                    DAIF.get(),
                    ctx
                );
                panic!("GIC unhandled SGI PPI")
            }
        }
        None => {
            warn!(
                "current_el_spx_irq, thread [{}], el{}, irq {}, daif: {:x}\n ctx on sp {:p}\n",
                TPIDRRO_EL0.get(),
                crate::arch::Arch::curent_privilege(),
                irq.unwrap(),
                DAIF.get(),
                ctx
            );
            panic!("GIC unknown irq")
        }
    }
    if irq.is_some() {}
    // debug!(
    //     "current_el_spx_irq call pop_context, cur_sp {:x}",
    //     core.current_sp()
    // );
}

#[no_mangle]
unsafe extern "C" fn current_el_sp0_synchronous(ctx: *mut ContextFrame) {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    let tid = TPIDRRO_EL0.get();
    panic!(
        "current_el_sp0_synchronous on Thread {}\nEC {:#X} ESR_EL1 {:#x}\n{}",
        tid,
        ec,
        ESR_EL1.get(),
        ctx.read()
    );
}

#[no_mangle]
unsafe extern "C" fn current_el_sp0_irq(ctx: *mut ContextFrame) {
    warn!(
        "current_el_sp0_irq, thread [{}], el{}, irq {}, daif: {:x}\n ctx on user_sp {:p}\n",
        TPIDRRO_EL0.get(),
        crate::arch::Arch::curent_privilege(),
        InterruptController::fetch().unwrap(),
        DAIF.get(),
        ctx
    );
    println!("{}", ctx.read());
    loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_serror(ctx: *mut ContextFrame) {
    panic!("current_el_spx_serror\n{}", ctx.read());
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(ctx: *mut ContextFrame) {
    let core_id = crate::arch::Arch::core_id();
    let tid = crate::libs::thread::current_thread_id();
    panic!(
        "core {} T[{}] lower_aarch64_synchronous\n {}",
        core_id,
        tid,
        ctx.read()
    );
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(ctx: *mut ContextFrame) {
    let core_id = crate::arch::Arch::core_id();

    panic!(
        "core {} lower_aarch64_irq EL{} \n {}",
        core_id,
        crate::arch::Arch::curent_privilege(),
        ctx.read()
    );
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(ctx: *mut ContextFrame) {
    let core_id = crate::arch::Arch::core_id();
    panic!("core {} lower_aarch64_serror\n {}", core_id, ctx.read());
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
