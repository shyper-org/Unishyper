use tock_registers::interfaces::{Readable, ReadWriteable};

use super::sbi::{sbi_call, SBI_EID_TIMER, SBI_FID_SET_TIMER};

const TIMER_DEFAULT_COUNT: usize = 250000;

const TIMER_TICK_MS: u64 = 100;

#[allow(dead_code)]
pub const TIMER_TICK_US: u64 = TIMER_TICK_MS * 1000;

pub fn next() {
    let _ = sbi_call(
        SBI_EID_TIMER,
        SBI_FID_SET_TIMER,
        counter() + TIMER_DEFAULT_COUNT,
        0,
        0,
    );
}

// NOTE: timer frequency can be obtained from FDT
// 	cpus {
// 		#address-cells = <0x01>;
// 		#size-cells = <0x00>;
// 		timebase-frequency = <0x989680>;
#[cfg(not(feature = "k210"))]
const TIMER_FREQUENCY: usize = 0x989680;

#[cfg(feature = "k210")]
const TIMER_FREQUENCY: usize = 7800000;

pub fn frequency() -> usize {
    TIMER_FREQUENCY
}

pub fn counter() -> usize {
    riscv::regs::TIME.get() as usize
}

pub fn init() {
    next();
    use riscv::regs::SIE;
    SIE.modify(SIE::STIE::SET);
}

pub fn current_cycle() -> usize {
    0
}

#[allow(dead_code)]
const TIMER_SEC_TO_MS: u64 = 1000;
#[allow(dead_code)]
const TIMER_SEC_TO_US: u64 = 1000_000;

pub fn timestamp_sec() -> u64 {
    const NSEC_PER_SEC: u64 = 1000_000_000;
    const GOLDFISH_MMIO_BASE: usize = 0xffff_ffff_0000_0000 + 0x101000;
    let low = unsafe { (GOLDFISH_MMIO_BASE as *mut u32).read() as u64 };
    let high = unsafe { ((GOLDFISH_MMIO_BASE + 4) as *mut u32).read() as u64 };
    ((high << 32) | low) / NSEC_PER_SEC
}

pub fn timestamp_us() -> u64 {
    timestamp_sec() * TIMER_SEC_TO_US
}
