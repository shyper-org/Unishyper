use core::fmt;
use core::iter::Step;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::arch::{PAGE_SIZE, MAX_PAGE_NUMBER};
use crate::libs::traits::Address;
use crate::mm::address::PAddr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}

impl Frame {
    pub const fn start_address(&self) -> PAddr {
        PAddr::new_canonical(self.number * PAGE_SIZE)
    }

    #[inline(always)]
    pub const fn number(&self) -> usize {
        self.number
    }

    pub const fn containing_address(addr: PAddr) -> Frame {
        Frame {
            number: addr.value() / PAGE_SIZE,
        }
    }
    pub fn zero(&self) {
        let dst = self.start_address().value().pa2kva();
        unsafe {
            core::ptr::write_bytes(dst as *mut u8, 0, PAGE_SIZE);
        }
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            concat!(stringify!(Frame), "(PAddr: 0x{:016x})"),
            self.start_address()
        )
    }
}

impl Add<usize> for Frame {
    type Output = Frame;
    fn add(self, rhs: usize) -> Frame {
        // cannot exceed max page number (which is also max frame number)
        Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}

impl AddAssign<usize> for Frame {
    fn add_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}

impl Sub<usize> for Frame {
    type Output = Frame;
    fn sub(self, rhs: usize) -> Frame {
        Frame {
            number: self.number.saturating_sub(rhs),
        }
    }
}

impl SubAssign<usize> for Frame {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: self.number.saturating_sub(rhs),
        };
    }
}

impl Step for Frame {
    #[inline]
    fn steps_between(start: &Frame, end: &Frame) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::forward_checked(start.number, count).map(|n| Frame { number: n })
    }
    #[inline]
    fn backward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::backward_checked(start.number, count).map(|n| Frame { number: n })
    }
}
