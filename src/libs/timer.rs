#[allow(dead_code)]
pub const TIMER_SEC_TO_MS: usize = 1000;
#[allow(dead_code)]
pub const TIMER_SEC_TO_US: usize = 1000_000;
#[allow(dead_code)]
pub const TIMER_SEC_TO_NS: usize = 1000_000_000;

//Todo: refactor these methods into different architectures.

pub fn interrupt() {
    // debug!("timer interrupt");
    crate::drivers::timer::next();
    crate::libs::thread::handle_blocked_threads();
    crate::libs::thread::handle_exit_threads();
    // crate::libs::thread::thread_yield();
}

#[allow(dead_code)]
pub fn current_cycle() -> usize {
    crate::drivers::timer::current_cycle()
}

#[cfg(feature = "std")]
pub(crate) const CLOCK_REALTIME: u64 = 1;
#[cfg(feature = "std")]
pub(crate) const CLOCK_MONOTONIC: u64 = 4;

#[allow(dead_code)]
/// Get current time in nanosecond(10 ^ -9 second).
pub fn current_ns() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count * TIMER_SEC_TO_NS / freq
}

#[allow(dead_code)]
/// Get current time in microsecond(10 ^ -6 second).
pub fn current_us() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count * TIMER_SEC_TO_US / freq
}

/// Get current time in millisecond(10 ^ -3 second).
pub fn current_ms() -> usize {
    let count = crate::drivers::timer::counter();
    let freq: usize = crate::drivers::timer::frequency();
    count * TIMER_SEC_TO_MS / freq
}

#[allow(dead_code)]
/// Get current time in second.
pub fn current_sec() -> usize {
    let count = crate::drivers::timer::counter();
    let freq = crate::drivers::timer::frequency();
    count / freq
}

use core::num::NonZeroUsize;
static mut BOOT_TIME: Option<NonZeroUsize> = None;

/// Get shyper system boot time in microsecond(10 ^ -6 second).
/// PENDING: RTC only has a second level precision, see init fn below.
pub fn boot_time() -> usize {
    unsafe {
        match BOOT_TIME {
            Some(t) => t.get(),
            None => 0,
        }
    }
}

pub fn init() {
    info!(
        "Unishyper starts at [{} (UTC)]",
        rtc_time64_to_tm(crate::drivers::timer::timestamp_sec() as u64)
    );
    let boot_time = crate::drivers::timer::timestamp_us() as usize - current_us();
    if boot_time > 0 {
        unsafe {
            BOOT_TIME = Some(boot_time.try_into().unwrap());
        }
    }
}

pub mod time {
    use core::fmt::{Display, Formatter};

    /// same as `struct rtc_time` in linux kernel
    #[derive(Default)]
    pub struct RtcTime {
        pub sec: i32,
        pub min: i32,
        pub hour: i32,
        pub mday: i32,
        pub mon: i32,
        pub year: i32,
        pub wday: i32,
        pub yday: i32,
        pub isdst: i32,
    }

    impl Display for RtcTime {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                self.year + 1900,
                self.mon + 1,
                self.mday,
                self.hour,
                self.min,
                self.sec
            )
        }
    }
}

use time::RtcTime;

fn rtc_time64_to_tm(time: u64) -> RtcTime {
    let leaps_thru_end_of = |y: i32| (y) / 4 - (y) / 100 + (y) / 400;
    let is_leap_year = |y: i32| ((y % 4 == 0) && (y % 100 != 0)) || (y % 400 == 0);
    let rtc_month_days = |month: i32, year: i32| -> i32 {
        const RTC_DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        RTC_DAYS_IN_MONTH[month as usize] as i32
            + if is_leap_year(year) && month == 1 {
                1
            } else {
                0
            }
    };
    let mut r = RtcTime::default();
    let mut days: i32 = (time / 86400) as i32;
    let mut secs: i32 = (time - (days as u64) * 86400) as i32;
    r.wday = ((days + 4) % 7) as i32;
    let mut year = 1970 + days / 365;
    days -= (year - 1970) * 365 + leaps_thru_end_of(year - 1) - leaps_thru_end_of(1970 - 1);
    if days < 0 {
        year -= 1;
        days += 365 + if is_leap_year(year) { 1 } else { 0 };
    }
    r.year = (year - 1900) as i32;
    r.yday = (days + 1) as i32;
    let mut month = 0;
    loop {
        if month == 12 {
            break;
        }
        let newdays = days - rtc_month_days(month, year);
        if newdays < 0 {
            break;
        }
        days = newdays;
        month += 1;
    }
    r.mon = month as i32;
    r.mday = (days + 1) as i32;
    r.hour = (secs / 3600) as i32;
    secs -= r.hour * 3600;
    r.min = (secs / 60) as i32;
    r.sec = (secs - r.min * 60) as i32;
    r.isdst = 0;
    r
}
