use alloc::collections::VecDeque;
use core::ops::Range;

use spin::{Mutex, Once};

use crate::arch::*;
use crate::lib::traits::*;

use super::Region;

pub type Error = usize;

// struct PPAllocator;
//
// unsafe impl Allocator for PPAllocator {
//   fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//     Global.allocate(layout)
//   }
//
//   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//     Global.deallocate(ptr, layout)
//   }
// }

struct PagePool {
    free: VecDeque<usize>,
}

impl PagePool {
    pub fn init(&mut self, range: Range<usize>) {
        assert_eq!(range.start % PAGE_SIZE, 0);
        assert_eq!(range.end % PAGE_SIZE, 0);
        unsafe {
            core::ptr::write_bytes(range.start as *mut u8, 0, range.len());
        }
        for pa in range.step_by(PAGE_SIZE) {
            self.free.push_back(pa);
        }
    }

    // Todo: we need to organize free pages better.
    pub fn allocate_pages(&mut self, num: usize) -> Result<Region, Error> {
        assert!(num > 0,"try to allocate zero page!");
        let p = self.free.pop_front().unwrap();
        let mut pa = p;
        // debug!("allocate_pages get pa 0x{:x}", p);
        if num > 1 {
            for _ in 1..num {
                let next_pa = self.free.pop_front().unwrap();
                if pa + PAGE_SIZE != next_pa {
                    panic!("allocate_pages no more free continues mem region pa {:x}!", pa);
                }
                // debug!("allocate_pages get next_pa 0x{:x}", next_pa);
                pa = next_pa;
            }
        }
        return Ok(Region::new(p, num * PAGE_SIZE))
    }

    pub fn free(&mut self, pa: usize) -> Result<(), Error> {
        // debug!(" free pa {:x}", pa);
        self.free.push_back(pa);
        Ok(())
    }
}

static PAGE_POOL: Once<Mutex<PagePool>> = Once::new();

fn page_pool() -> &'static Mutex<PagePool> {
    PAGE_POOL.get().unwrap()
}

pub fn init() {
    let range = super::config::paged_range();
    println!(
        "Paged range: pa [{:x} - {:x}] kva [{:x} - {:x}]",
        range.start,
        range.end,
        range.start.pa2kva(),
        range.end.pa2kva()
    );
    PAGE_POOL.call_once(|| {
        Mutex::new(PagePool {
            free: VecDeque::new(),
        })
    });
    let mut pool = page_pool().lock();
    pool.init(range);
}

pub fn pages_alloc(num: usize) -> Result<Region, Error> {
    let mut pool = page_pool().lock();
    // debug!("pages_alloc num {}",num);
    pool.allocate_pages(num)
}

pub fn page_free(pa: usize) -> Result<(), Error> {
    let mut pool = page_pool().lock();
    pool.free(pa)
}
