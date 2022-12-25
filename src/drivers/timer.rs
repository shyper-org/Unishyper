use cortex_a::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTV_CTL_EL0, CNTV_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

const TIMER_TICK_MS: u64 = 100;

#[allow(dead_code)]
pub const TIMER_TICK_US: u64 = TIMER_TICK_MS * 1000;

pub fn next() {
    let freq = CNTFRQ_EL0.get();
    let count = TIMER_TICK_MS * freq / 1000;
    CNTV_TVAL_EL0.set(count);
    CNTV_CTL_EL0.write(CNTV_CTL_EL0::ENABLE.val(1) + CNTV_CTL_EL0::IMASK.val(0));
}

/// Clock frequency. Indicates the system counter clock frequency, in Hz.
pub fn frequency() -> usize {
    CNTFRQ_EL0.get() as usize
}

pub fn counter() -> usize {
    CNTPCT_EL0.get() as usize
}

pub fn init() {
    next();
}
