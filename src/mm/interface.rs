use crate::mm::paging::{Entry, EntryAttribute};

pub trait PageTableEntryAttrTrait {
    fn writable(&self) -> bool;
    fn k_executable(&self) -> bool;
    fn u_executable(&self) -> bool;
    fn u_readable(&self) -> bool;
    fn u_copy_on_write(&self) -> bool;
    fn u_shared(&self) -> bool;
    fn device(&self) -> bool;
    fn block(&self) -> bool;
    fn set_block(&self) -> Self;
    fn copy_on_write(&self) -> bool;

    fn new(
        writable: bool,
        user: bool,
        device: bool,
        k_executable: bool,
        u_executable: bool,
        copy_on_write: bool,
        shared: bool,
        block: bool,
    ) -> Self;
    fn kernel_device() -> Self;
    fn user_default() -> Self;
    fn user_2mb() -> Self;
    fn user_readonly() -> Self;
    fn user_executable() -> Self;
    fn user_data() -> Self;
    fn user_device() -> Self;
    fn filter(&self) -> Self;
}

#[cfg(target_arch = "x86_64")]
pub trait PageTableEntryAttrZoneTrait {
    fn set_zone(&self, zone_id: u16) -> Self;
    fn get_zone_key(&self) -> u16;
}

pub type Error = usize;

pub trait PageTableTrait {
    fn base_pa(&self) -> usize;
    fn map(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error>;
    fn map_2mb(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error>;
    fn unmap(&mut self, va: usize);
    fn unmap_2mb(&mut self, va: usize);
    // fn insert_page(
    //     &self,
    //     va: usize,
    //     user_frame: crate::mm::Frame,
    //     attr: EntryAttribute,
    // ) -> Result<(), Error>;
    fn lookup_entry(&self, va: usize) -> Option<(Entry, MapGranularity)>;
    fn lookup_page(&self, va: usize) -> Option<Entry>;
    // fn remove_page(&self, va: usize) -> Result<(), Error>;
    fn recursive_map(&self, va: usize);
}

use crate::arch::PAGE_SHIFT;
use crate::arch::page_table::PAGE_TABLE_L2_SHIFT;
use crate::arch::page_table::PAGE_TABLE_L1_SHIFT;
#[allow(unused)]
#[derive(Debug)]
pub enum MapGranularity {
    /// Mapped by 4KB page.
    Page4KB = 1 << PAGE_SHIFT,
    /// Mapped by 2MB page.
    Page2MB = 1 << PAGE_TABLE_L2_SHIFT,
    /// Mapped by 1GB page, current unused.
    #[allow(unused)]
    Page1GB = 1 << PAGE_TABLE_L1_SHIFT,
}

impl core::convert::Into<usize> for MapGranularity {
    #[inline]
    fn into(self) -> usize {
        match self {
            MapGranularity::Page4KB => 1 << PAGE_SHIFT,
            MapGranularity::Page2MB => 1 << PAGE_TABLE_L2_SHIFT,
            MapGranularity::Page1GB => 1 << PAGE_TABLE_L1_SHIFT,
        }
    }
}
