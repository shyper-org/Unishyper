// use spin::barrier::Barrier;

use crate::arch::BOARD_CORE_NUMBER;
use core::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn barrier() {
    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let next_count = round_up(count + 1, BOARD_CORE_NUMBER);
    loop {
        if COUNT.load(Ordering::Acquire) >= next_count {
            break;
        }
    }
}

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
    (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
    addr & !(n - 1)
}

use crate::arch::irq;

/// `irqsave` guarantees that the call of the closure
/// will be not disturbed by an interrupt
#[inline]
pub fn irqsave<F, R>(f: F) -> R
where
	F: FnOnce() -> R,
{
	let irq = irq::nested_disable();
	let ret = f();
	irq::nested_enable(irq);
	ret
}