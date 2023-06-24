const TIMER_TICK_MS: u64 = 100;
#[allow(dead_code)]
pub const TIMER_TICK_US: u64 = TIMER_TICK_MS * 1000;

use core::sync::atomic::{AtomicU64, Ordering};

static INIT_TICK: AtomicU64 = AtomicU64::new(0);
static TSC_FREQUENCY_MHZ: AtomicU64 = AtomicU64::new(2600);
static TSC_FREQUENCY_HZ: AtomicU64 = AtomicU64::new(2600_000_000);

pub fn next() {}

/// Clock frequency. Indicates the system counter clock frequency, in Hz.
pub fn frequency() -> usize {
    TSC_FREQUENCY_HZ.load(Ordering::Relaxed) as usize
}

pub fn counter() -> usize {
    (unsafe { core::arch::x86_64::_rdtsc() } - INIT_TICK.load(Ordering::Relaxed)) as usize
}

pub fn init() {
    let cpuid = raw_cpuid::CpuId::new();
    // Detect from CpuId info.
    if let Some(freq) = cpuid
        .get_processor_frequency_info()
        .map(|info| info.processor_base_frequency())
    {
        if freq > 0 {
            info!("Got Processor frequency from CPUID: {} MHz", freq);
            TSC_FREQUENCY_MHZ.store(freq as u64, Ordering::Relaxed);
            TSC_FREQUENCY_HZ.store(freq as u64 * 1000 * 1000, Ordering::Relaxed);
        } else {
            warn!("failed to get frequency from CPUID");
        }
    // Detect from CPU tsc info.
    } else if let Some(freq) = cpuid
        .get_tsc_info()
        .map(|tsc| tsc.tsc_frequency().unwrap_or(0))
    {
        if freq > 0 {
            info!("Got Processor frequency from TSC INFO: {} MHz", freq);
            TSC_FREQUENCY_MHZ.store(freq as u64, Ordering::Relaxed);
            TSC_FREQUENCY_HZ.store(freq as u64 * 1000 * 1000, Ordering::Relaxed);
        } else {
            warn!("failed to get frequency from TSC INFO");
        }
    // Detect from CPU brand string.
    } else if let Some(processor_brand) = cpuid.get_processor_brand_string() {
        let brand_string = processor_brand.as_str();
        info!("CPU processor string {}", brand_string);
        let ghz_find = brand_string.find("GHz");

        if let Some(ghz_find) = ghz_find {
            let index = ghz_find - 4;
            let thousand_char = brand_string.chars().nth(index).unwrap();
            let decimal_char = brand_string.chars().nth(index + 1).unwrap();
            let hundred_char = brand_string.chars().nth(index + 2).unwrap();
            let ten_char = brand_string.chars().nth(index + 3).unwrap();

            if let (Some(thousand), '.', Some(hundred), Some(ten)) = (
                thousand_char.to_digit(10),
                decimal_char,
                hundred_char.to_digit(10),
                ten_char.to_digit(10),
            ) {
                let freq = (thousand * 1000 + hundred * 100 + ten * 10) as u16;
                info!(
                    "Got Processor frequency from CPU brand string: {} MHz",
                    freq
                );
                TSC_FREQUENCY_MHZ.store(freq as u64, Ordering::Relaxed);
                TSC_FREQUENCY_HZ.store(freq as u64 * 1000 * 1000, Ordering::Relaxed);
            }
        }
    } else {
        let freq = TSC_FREQUENCY_MHZ.load(Ordering::Relaxed);
        warn!("Could not determine the processor frequency! Guess a frequency of {freq}MHZ!");
    }
    INIT_TICK.store(unsafe { core::arch::x86_64::_rdtsc() }, Ordering::Relaxed);
}

pub fn timestamp_us() -> u64 {
    let rtc = super::rtc::Rtc::new();
    rtc.get_microseconds_since_epoch()
}

pub fn timestamp_sec() -> u64 {
    timestamp_us() / 1000_000u64
}

pub fn current_cycle() -> usize {
    (unsafe { core::arch::x86_64::_rdtsc() } - INIT_TICK.load(Ordering::Relaxed)) as usize
}
