// use spin::barrier::Barrier;
#[cfg(feature = "smp")]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "smp")]
static COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn barrier() {
    #[cfg(feature = "smp")]
    {
        use crate::board::BOARD_CORE_NUMBER;
        let count = COUNT.fetch_add(1, Ordering::AcqRel);
        let next_count = round_up(count + 1, BOARD_CORE_NUMBER);
        loop {
            if COUNT.load(Ordering::Acquire) >= next_count {
                break;
            }
        }
    }
}

#[inline(always)]
#[allow(dead_code)]
pub fn round_up(addr: usize, n: usize) -> usize {
    (addr + n - 1) & !(n - 1)
}

#[inline(always)]
#[allow(dead_code)]
pub fn round_down(addr: usize, n: usize) -> usize {
    addr & !(n - 1)
}

use crate::arch::irq;

// static mut irqsave_mark: usize = 0;
/// `irqsave` guarantees that the call of the closure
/// will be not disturbed by an interrupt
#[inline]
pub fn irqsave<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // unsafe {
    //     irqsave_mark += 1;
    //     println!("irqsave nested_disable on ({}) to ({})", irqsave_mark-1, irqsave_mark);
    // }
    let irq = irq::nested_disable();
    let ret = f();
    irq::nested_enable(irq);
    // unsafe {
    //     println!("irqsave nested_enable on ({}) to ({})", irqsave_mark, irqsave_mark -1);
    //     irqsave_mark -= 1;
    // }
    ret
}
