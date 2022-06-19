use crate::arch::PAGE_SIZE;
use crate::lib::traits::*;

use super::Addr;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Region {
    pa: usize,
    size: usize,
}

impl Region {
    pub fn new(pa: usize, size: usize) -> Self {
        assert_eq!(pa % PAGE_SIZE, 0);
        Region { pa, size }
    }

    pub fn kva(&self) -> usize {
        self.pa.pa2kva()
    }

    pub fn pa(&self) -> usize {
        self.pa
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn zero(&self) {
        unsafe {
            core::ptr::write_bytes(self.kva() as *mut u8, 0, self.size);
        }
    }

    pub fn addr(&self) -> Addr {
        Addr(self.kva())
    }
}



impl Drop for Region {
    fn drop(&mut self) {
        // debug!(
        //     "drop region {:016x} to {:016x}",
        //     self.pa,
        //     self.pa + self.size - 1
        // );
        for pa in (self.pa..self.pa + self.size).step_by(PAGE_SIZE) {
            super::page_pool::page_free(pa).expect("physical page drop failed");
        }
    }
}
