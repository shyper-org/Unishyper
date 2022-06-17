pub mod config;
pub mod heap;
mod page_frame;
pub mod page_pool;

pub use self::page_frame::*;

use alloc::vec::Vec;
use core::alloc::AllocError;

use crate::arch::PAGE_SIZE;
use crate::lib::thread::current_thread;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Addr(pub usize);

impl Addr {
    /// Convert to `usize`
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for Addr {
    fn from(num: usize) -> Self {
        Addr(num)
    }
}

impl From<Vec<PhysicalFrame>> for Addr {
    fn from(pages: Vec<PhysicalFrame>) -> Self {
        assert!(pages.len() > 0);
        Addr(pages[0].kva())
    }
}

#[allow(clippy::clippy::from_over_into)]
impl Into<usize> for Addr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

pub fn allocate(size: usize) -> Result<Addr, AllocError> {
    assert!(size > 0);
    assert_eq!(
        size % PAGE_SIZE,
        0,
        "Size {:#X} is not a multiple of {:#X}",
        size,
        size % PAGE_SIZE,
    );

    let t = match current_thread() {
        Ok(t) => t,
        Err(_) => {
            panic!("no current thread!");
        }
    };

    let frames =
        page_pool::pages_alloc(size / PAGE_SIZE).expect("failed to allocate physical frame");

    let addr: Addr = frames.into();

    t.add_address_space(addr, frames.as_ref());

    Ok(addr)
}

pub fn deallocate(address: Addr) {
    assert!(
        address >= crate::mm::config::kernel_end_address(),
        "address {:#X} is not >= KERNEL_END_ADDRESS",
        address.as_usize()
    );

    let t = match current_thread() {
        Ok(t) => t,
        Err(_) => {
            panic!("no current thread!");
        }
    };

    t.free_address_space(address);
}
