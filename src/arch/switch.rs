core::arch::global_asm!(include_str!("context.S"));

use crate::{arch::ContextFrame, libs::stack::get_core_stack};

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
    // debug!("save_context return");
}

#[no_mangle]
unsafe extern "C" fn set_cpu_context(ctx: *mut ContextFrame) {
    // debug!("set_cpu_context\n {}", ctx.read());
    let core = crate::libs::cpu::cpu();
    core.set_context(ctx);
    // debug!("core set_context success");
    crate::libs::thread::thread_schedule();
    core.clear_context();
}
