pub mod config;
pub mod heap;
mod mem_region;
pub mod page_pool;

use alloc::collections::BTreeMap;
use spin::Mutex;

pub use self::mem_region::*;

use crate::arch::PAGE_SIZE;
use crate::lib::thread::current_thread;
use crate::lib::traits::Address;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Addr(pub usize);

impl Addr {
    pub fn to_pa(self) -> usize {
        self.0.kva2pa()
    }
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
    fn from(addr: usize) -> Self {
        Addr(addr)
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

static GLOBAL_MM_MAP: Mutex<BTreeMap<Addr, Region>> = Mutex::new(BTreeMap::new());

pub fn allocate(size: usize) -> Addr {
    assert!(size > 0);
    assert_eq!(
        size % PAGE_SIZE,
        0,
        "Size {:#X} is not a multiple of {:#X}",
        size,
        size % PAGE_SIZE,
    );

    let region =
        page_pool::pages_alloc(size / PAGE_SIZE).expect("failed to allocate physical frame");

    // debug!("allocate region start 0x{:x} size 0x{:x}", region.kva(), region.size());

    let addr = region.addr();

    match current_thread() {
        Ok(t) => {
            debug!(
                "thread {} alloc size 0x{:x} pages_num {} region start 0x{:x} size 0x{:x}",
                t.tid(),
                size,
                size / PAGE_SIZE,
                region.kva(),
                region.size()
            );
            t.add_address_space(addr, region);
        }
        Err(_) => {
            // debug!(
            //     "thread GLOBAL alloc size 0x{:x} pages_num {} region start 0x{:x} size 0x{:x}",
            //     size,
            //     size / PAGE_SIZE,
            //     region.kva(),
            //     region.size()
            // );
            GLOBAL_MM_MAP.lock().insert(addr, region);
        }
    };

    addr
}

pub fn deallocate(address: Addr) {
    assert!(
        address >= crate::mm::config::kernel_end_address(),
        "address {:#X} is not >= KERNEL_END_ADDRESS",
        address.as_usize()
    );

    match current_thread() {
        Ok(t) => {
            t.free_address_space(address);
        },
        Err(_) => {
            debug!("no current thread!");
        }
    };

    debug!("deallocate region addr start 0x{:x}", address.as_usize());
}
