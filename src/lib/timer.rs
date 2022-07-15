#[allow(dead_code)]
const TIMER_SEC_TO_MS: usize = 1000;
#[allow(dead_code)]
const TIMER_SEC_TO_US: usize = 1000000;

#[allow(dead_code)]
pub fn current_us() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count * TIMER_SEC_TO_US / freq
}

#[allow(dead_code)]
pub fn current_ms() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count * TIMER_SEC_TO_MS / freq
}

#[allow(dead_code)]
pub fn current_sec() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count / freq
}

pub fn interrupt() {
  crate::drivers::timer::next();
//   trace!("timer interrupt");
  crate::lib::thread::handle_blocked_threads();
  crate::lib::cpu::cpu().schedule();
}

#[allow(dead_code)]
pub fn current_cycle() -> usize {
    let r;
    unsafe {
        core::arch::asm!("mrs {}, pmccntr_el0", out(reg) r);
    }
    r
}
