use core::fmt;
use core::iter::Step;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::arch::{PAGE_SIZE, MAX_PAGE_NUMBER};
use crate::mm::address::VAddr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    pub number: usize,
}

impl Page {
    pub const fn start_address(&self) -> VAddr {
        VAddr::new_canonical(self.number * PAGE_SIZE)
    }

    #[inline(always)]
    pub const fn number(&self) -> usize {
        self.number
    }

    pub const fn containing_address(addr: VAddr) -> Page {
        Page {
            number: addr.value() / PAGE_SIZE,
        }
    }
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            concat!(stringify!(Page), "(VAddr: 0x{:016x})"),
            self.start_address()
        )
    }
}

impl Add<usize> for Page {
    type Output = Page;
    fn add(self, rhs: usize) -> Page {
        // cannot exceed max page number (which is also max frame number)
        Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}

impl AddAssign<usize> for Page {
    fn add_assign(&mut self, rhs: usize) {
        *self = Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}

impl Sub<usize> for Page {
    type Output = Page;
    fn sub(self, rhs: usize) -> Page {
        Page {
            number: self.number.saturating_sub(rhs),
        }
    }
}

impl SubAssign<usize> for Page {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Page {
            number: self.number.saturating_sub(rhs),
        };
    }
}

impl Step for Page {
    #[inline]
    fn steps_between(start: &Page, end: &Page) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Page, count: usize) -> Option<Page> {
        Step::forward_checked(start.number, count).map(|n| Page { number: n })
    }
    #[inline]
    fn backward_checked(start: Page, count: usize) -> Option<Page> {
        Step::backward_checked(start.number, count).map(|n| Page { number: n })
    }
}
