pub mod config;
pub mod heap;
mod mem_region;
mod page_frame;
pub mod page_pool;

pub use self::mem_region::*;
pub use self::page_frame::*;

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

    /// Convert to mutable pointer.
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Convert to pointer.
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
}

impl From<usize> for Addr {
    fn from(num: usize) -> Self {
        Addr(num)
    }
}

impl From<Region> for Addr {
    fn from(region: Region) -> Self {
        assert!(region.size() > 0);
        Addr(region.kva())
    }
}

#[allow(clippy::clippy::from_over_into)]
impl Into<usize> for Addr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

pub fn allocate(size: usize) -> Addr {
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

    // debug!("thread {} alloc size 0x{:x} pages_num {}", t.tid(), size, size / PAGE_SIZE);

    let region =
        page_pool::pages_alloc(size / PAGE_SIZE).expect("failed to allocate physical frame");

    debug!(
        "allocate region start 0x{:x} size 0x{:x}",
        region.kva(),
        region.size()
    );

    let addr = region.addr();
    t.add_address_space(addr, region);

    addr
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

    debug!("deallocate region addr start 0x{:x}", address.as_usize());

    t.free_address_space(address);
}
