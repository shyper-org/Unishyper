use core::fmt;
use core::cmp::{min, max};
use core::ops::{Deref, DerefMut, RangeInclusive};

use crate::arch::PAGE_SIZE;
use crate::mm::address::PAddr;
use crate::mm::frame_allocator::frame::Frame;

#[derive(Clone, PartialEq, Eq)]
pub struct FrameRange(RangeInclusive<Frame>);

#[allow(unused)]
impl FrameRange {
    pub const fn new(start: Frame, end: Frame) -> FrameRange {
        FrameRange(RangeInclusive::new(start, end))
    }

    pub const fn empty() -> FrameRange {
        FrameRange::new(Frame { number: 1 }, Frame { number: 0 })
    }

    pub fn from_phys_addr(starting_addr: PAddr, size_in_bytes: usize) -> FrameRange {
        assert!(size_in_bytes > 0);
        let start = Frame::containing_address(starting_addr);
        // The end bound is inclusive, hence the -1. Parentheses are needed to avoid overflow.
        let end = Frame::containing_address(starting_addr + (size_in_bytes - 1));
        FrameRange::new(start, end)
    }

    pub const fn start_address(&self) -> PAddr {
        self.0.start().start_address()
    }

    pub const fn size_in_frames(&self) -> usize {
        // add 1 because it's an inclusive range
        (self.0.end().number + 1).saturating_sub(self.0.start().number)
    }

    /// Returns the size of this range in number of bytes.
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_frames() * PAGE_SIZE
    }

    pub fn contains_address(&self, addr: PAddr) -> bool {
        self.0.contains(&Frame::containing_address(addr))
    }

    pub fn offset_of_address(&self, addr: PAddr) -> Option<usize> {
        if self.contains_address(addr) {
            Some(addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    pub fn address_at_offset(&self, offset: usize) -> Option<PAddr> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        } else {
            None
        }
    }

    pub fn to_extended(&self, to_include: Frame) -> FrameRange {
        // if the current range was empty, return a new range containing only the given page/frame
        if self.is_empty() {
            return FrameRange::new(to_include.clone(), to_include);
        }
        let start = core::cmp::min(self.0.start(), &to_include);
        let end = core::cmp::max(self.0.end(), &to_include);
        FrameRange::new(start.clone(), end.clone())
    }

    pub fn overlap(&self, other: &FrameRange) -> Option<FrameRange> {
        let starts = max(*self.start(), *other.start());
        let ends = min(*self.end(), *other.end());
        if starts <= ends {
            Some(FrameRange::new(starts, ends))
        } else {
            None
        }
    }
}
impl fmt::Debug for FrameRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for FrameRange {
    type Target = RangeInclusive<Frame>;
    fn deref(&self) -> &RangeInclusive<Frame> {
        &self.0
    }
}
impl DerefMut for FrameRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Frame> {
        &mut self.0
    }
}
impl IntoIterator for FrameRange {
    type Item = Frame;
    type IntoIter = RangeInclusive<Frame>;
    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}
