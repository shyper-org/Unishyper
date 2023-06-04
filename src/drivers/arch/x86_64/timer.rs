const TIMER_TICK_MS: u64 = 100;
#[allow(dead_code)]
pub const TIMER_TICK_US: u64 = TIMER_TICK_MS * 1000;

#[allow(dead_code)]
const TSC_FREQUENCY: u16 = 2600;

pub fn next() {}

/// Clock frequency. Indicates the system counter clock frequency, in Hz.
pub fn frequency() -> usize {
    TSC_FREQUENCY as usize
}

unsafe fn get_timestamp_rdtsc() -> u64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        let value = core::arch::x86_64::_rdtsc();
        core::arch::x86_64::_mm_lfence();
        value
    }
}

pub fn counter() -> usize {
    unsafe { get_timestamp_rdtsc() as usize }
}

pub fn init() {
    next();
}

#[allow(dead_code)]
const TIMER_SEC_TO_MS: u64 = 1000;
#[allow(dead_code)]
const TIMER_SEC_TO_US: u64 = 1000_000;

pub fn timestamp_us() -> u64 {
    let rtc = super::rtc::Rtc::new();
    rtc.get_microseconds_since_epoch()
}

pub fn timestamp_sec() -> u64 {
    timestamp_us() / TIMER_SEC_TO_US
}

pub fn current_cycle() -> usize {
    0
}
