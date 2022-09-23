use crate::mm::paging::{Entry, EntryAttribute};

pub trait PageTableEntryAttrTrait {
    fn writable(&self) -> bool;
    fn k_executable(&self) -> bool;
    fn u_executable(&self) -> bool;
    fn u_readable(&self) -> bool;
    fn u_copy_on_write(&self) -> bool;
    fn u_shared(&self) -> bool;
    fn device(&self) -> bool;
    fn copy_on_write(&self) -> bool;

    fn new(
        writable: bool,
        user: bool,
        device: bool,
        k_executable: bool,
        u_executable: bool,
        copy_on_write: bool,
        shared: bool,
    ) -> Self;
    fn kernel_device() -> Self;
    fn user_default() -> Self;
    fn user_readonly() -> Self;
    fn user_executable() -> Self;
    fn user_data() -> Self;
    fn user_device() -> Self;
    fn filter(&self) -> Self;
}

pub type Error = usize;

pub trait PageTableTrait {
    fn new(directory: crate::mm::frame_allocator::AllocatedFrames) -> Self;
    fn base_pa(&self) -> usize;
    fn map(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error>;
    fn unmap(&self, va: usize);
    fn insert_page(
        &self,
        va: usize,
        user_frame: crate::mm::Frame,
        attr: EntryAttribute,
    ) -> Result<(), Error>;
    fn lookup_page(&self, va: usize) -> Option<Entry>;
    fn remove_page(&self, va: usize) -> Result<(), Error>;
    fn recursive_map(&self, va: usize);
}
