use alloc::vec::Vec;
use alloc::collections::VecDeque;
use core::ops::Range;

use spin::{Mutex, Once};

use crate::arch::*;
use crate::mm::PhysicalFrame;
use crate::lib::error::ERROR_OOM;

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

    pub fn allocate_page(&mut self) -> Result<PhysicalFrame, Error> {
        if let Some(pa) = self.free.pop_front() {
            Ok(PhysicalFrame::new(pa))
        } else {
            Err(ERROR_OOM)
        }
    }

    // Todo: we need to organize free pages better.
    pub fn allocate_pages(&mut self, num: usize) -> Result<Vec<PhysicalFrame>, Error> {
        let mut pages_queue: Vec<PhysicalFrame> = Vec::new();
        for _ in 0..num {
            let p = self.free.pop_front().unwrap();
            if pages_queue.len() > 0 {
                if pages_queue[pages_queue.len()-1].pa() != p {
                    return Err(ERROR_OOM)
                } 
            }
            pages_queue.push(PhysicalFrame::new(p));
        }

        return Ok(pages_queue);
    }

    pub fn free(&mut self, pa: usize) -> Result<(), Error> {
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
    PAGE_POOL.call_once(|| {
        Mutex::new(PagePool {
            free: VecDeque::new(),
        })
    });
    let mut pool = page_pool().lock();
    pool.init(range);
}

pub fn page_alloc() -> Result<PhysicalFrame, Error> {
    let mut pool = page_pool().lock();
    pool.allocate_page()
}

pub fn pages_alloc(num: usize) -> Result<Vec<PhysicalFrame>, Error> {
    let mut pool = page_pool().lock();
    pool.allocate_pages(num)
}

pub fn page_free(pa: usize) -> Result<(), Error> {
    let mut pool = page_pool().lock();
    pool.free(pa)
}
