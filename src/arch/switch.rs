core::arch::global_asm!(include_str!("context.S"));

use crate::arch::ContextFrame;

#[inline(always)]
pub fn switch_to() {
    // debug!("switch_to");
    extern "C" {
        fn save_context();
    }
    
    // debug!("save_context");
    unsafe { save_context();}
    // xxxxxxx
    // debug!("save_context return");
}

#[no_mangle]
unsafe extern "C" fn set_cpu_context(ctx: *mut ContextFrame) {
    debug!("set_cpu_context\n {}", ctx.read());
    let core = crate::lib::cpu::cpu();
    core.set_context(ctx);
    // debug!("core set_context success");
    crate::lib::thread::_thread_yield();
    core.clear_context();
    // debug!("core clear_context success");
}