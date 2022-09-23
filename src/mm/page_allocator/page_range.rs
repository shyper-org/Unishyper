use core::fmt;
use core::cmp::{min, max};
use core::ops::{Deref, DerefMut, RangeInclusive};

use crate::arch::PAGE_SIZE;
use crate::mm::address::VAddr;
use crate::mm::page_allocator::page::Page;

#[derive(Clone, PartialEq, Eq)]
pub struct PageRange(RangeInclusive<Page>);

impl PageRange {
    pub const fn new(start: Page, end: Page) -> PageRange {
        PageRange(RangeInclusive::new(start, end))
    }

    pub const fn empty() -> PageRange {
        PageRange::new(Page { number: 1 }, Page { number: 0 })
    }

    pub fn from_virt_addr(starting_addr: VAddr, size_in_bytes: usize) -> PageRange {
        assert!(size_in_bytes > 0);
        let start = Page::containing_address(starting_addr);
        // The end bound is inclusive, hence the -1. Parentheses are needed to avoid overflow.
        let end = Page::containing_address(starting_addr + (size_in_bytes - 1));
        PageRange::new(start, end)
    }

    pub const fn start_address(&self) -> VAddr {
        self.0.start().start_address()
    }

    pub const fn size_in_pages(&self) -> usize {
        // add 1 because it's an inclusive range
        (self.0.end().number() + 1).saturating_sub(self.0.start().number())
    }

    /// Returns the size of this range in number of bytes.
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_pages() * PAGE_SIZE
    }

    pub fn contains_address(&self, addr: VAddr) -> bool {
        self.0.contains(&Page::containing_address(addr))
    }

    pub fn offset_of_address(&self, addr: VAddr) -> Option<usize> {
        if self.contains_address(addr) {
            Some(addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    pub fn address_at_offset(&self, offset: usize) -> Option<VAddr> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        } else {
            None
        }
    }

    pub fn to_extended(&self, to_include: Page) -> PageRange {
        // if the current range was empty, return a new range containing only the given page/frame
        if self.is_empty() {
            return PageRange::new(to_include.clone(), to_include);
        }
        let start = core::cmp::min(self.0.start(), &to_include);
        let end = core::cmp::max(self.0.end(), &to_include);
        PageRange::new(start.clone(), end.clone())
    }

    pub fn overlap(&self, other: &PageRange) -> Option<PageRange> {
        let starts = max(*self.start(), *other.start());
        let ends = min(*self.end(), *other.end());
        if starts <= ends {
            Some(PageRange::new(starts, ends))
        } else {
            None
        }
    }
}
impl fmt::Debug for PageRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl Deref for PageRange {
    type Target = RangeInclusive<Page>;
    fn deref(&self) -> &RangeInclusive<Page> {
        &self.0
    }
}
impl DerefMut for PageRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Page> {
        &mut self.0
    }
}
impl IntoIterator for PageRange {
    type Item = Page;
    type IntoIter = RangeInclusive<Page>;
    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}
