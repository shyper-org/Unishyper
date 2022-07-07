core::arch::global_asm!(include_str!("context.S"));

use crate::{arch::ContextFrame, util::irqsave, lib::stack::get_core_stack};

#[inline(always)]
pub fn switch_to() {
    // debug!("switch_to");
    extern "C" {
        fn save_context(stack: usize);
    }

    let stack = get_core_stack();
    // debug!("save_context");
    unsafe {
        save_context(stack);
    }
    // xxxxxxx
    // debug!("save_context return");
}

#[no_mangle]
unsafe extern "C" fn set_cpu_context(ctx: *mut ContextFrame) {
    trace!("set_cpu_context\n {}", ctx.read());
    // irqsave(|| {
        let core = crate::lib::cpu::cpu();
        core.set_context(ctx);
        // debug!("core set_context success");
        crate::lib::thread::thread_schedule();
        core.clear_context();
    // });
    // debug!("core clear_context success");
}
