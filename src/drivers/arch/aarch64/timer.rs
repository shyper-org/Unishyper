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

pub fn current_cycle() -> usize {
    let r;
    unsafe {
        core::arch::asm!("mrs {}, pmccntr_el0", out(reg) r);
    }
    r
}

#[allow(dead_code)]
const TIMER_SEC_TO_MS: u64 = 1000;
#[allow(dead_code)]
const TIMER_SEC_TO_US: u64 = 1000_000;

#[cfg(feature = "qemu")]
pub fn timestamp_sec() -> u64 {
    const PL031_MMIO_BASE: usize = 0xFFFF_FF80_0000_0000 + 0x9010000;
    unsafe { (PL031_MMIO_BASE as *mut u32).read() as u64 }
}

#[cfg(not(feature = "qemu"))]
pub fn timestamp_sec() -> u64 {
    0
}

pub fn timestamp_us() -> u64 {
    timestamp_sec() * TIMER_SEC_TO_US
}
