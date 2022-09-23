use core::fmt;
use core::ops::Deref;

use crate::arch::page_table::page_table;
use crate::mm::interface::{PageTableEntryAttrTrait, PageTableTrait};

use crate::mm::page_allocator::{AllocatedPages, PageRange};

use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::paging::entry::EntryAttribute;
use crate::mm::frame_allocator;
use crate::mm::address::{PAddr, VAddr};

pub struct MappedRegion {
    pages: AllocatedPages,
    frames: AllocatedFrames,
    attribute: EntryAttribute,
}

impl fmt::Debug for MappedRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MappedRegion\n\tpages ({:?})\n\tframes({:?})\n\tattributes {:?}",
            self.pages, self.frames, self.attribute
        )
    }
}

impl Deref for MappedRegion {
    type Target = PageRange;
    fn deref(&self) -> &PageRange {
        self.pages.deref()
    }
}

impl MappedRegion {
    /// Returns an empty MappedRegion object that performs no allocation or mapping actions.
    /// Can be used as a placeholder, but will not permit any real usage.
    pub fn empty() -> MappedRegion {
        MappedRegion {
            pages: AllocatedPages::empty(),
            frames: AllocatedFrames::empty(),
            attribute: EntryAttribute::user_default(),
        }
    }

    /// Returns the attributes that describe this `MappedRegion` page table permissions.
    pub fn attribute(&self) -> EntryAttribute {
        self.attribute
    }

    /// Remove the virtual memory mapping represented by this `MappedRegion`.
    fn unmap(&mut self) {
        if self.size_in_pages() == 0 {
            return;
        }

        let page_table = crate::arch::page_table::page_table().lock();
        for page in self.pages.clone() {
            page_table.unmap(page.start_address().value());
        }
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        if self.size_in_pages() > 0 {
            trace!(
                "MappedRegion::drop(): unmapped MappedRegion {:?}, attribute: {:?}",
                &*self.pages,
                self.attribute
            );
        }
        self.unmap()
    }
}

pub fn map_allocated_pages(
    pages: AllocatedPages,
    attr: EntryAttribute,
) -> Result<MappedRegion, &'static str> {
    let frames = match frame_allocator::allocate_frames(pages.size_in_pages()) {
        Some(allocated_frames) => allocated_frames,
        None => {
            return Err("map_allocated_pages(): couldn't allocate new frame, out of memory");
        }
    };
    debug!(
        "map_allocated_pages(): {} pages:{} frames:{}",
        pages.size_in_pages(),
        pages.start().start_address(),
        frames.start().start_address()
    );

    let page_table = crate::arch::page_table::page_table().lock();
    for (page, frame) in pages
        .deref()
        .clone()
        .into_iter()
        .zip(frames.deref().clone().into_iter())
    {
        // Todo: unhandled error
        match page_table.map(
            page.start_address().value(),
            frame.start_address().value(),
            attr,
        ) {
            Ok(()) => continue,
            Err(_) => {
                return Err("page table map error");
            }
        }
    }

    Ok(MappedRegion {
        pages,
        frames,
        attribute: attr,
    })
}

/// Mapped allocated pages to target allocated frames.
/// Current this function may be useless.
#[allow(unused)]
pub fn map_allocated_pages_to(
    pages: AllocatedPages,
    frames: AllocatedFrames,
    attr: EntryAttribute,
) -> Result<MappedRegion, &'static str> {
    let pages_count = pages.size_in_pages();
    let frames_count = frames.size_in_frames();
    if pages_count != frames_count {
        error!(
            "map_allocated_pages_to(): pages {:?} count {} must equal frames {:?} count {}!",
            pages, pages_count, frames, frames_count
        );
        return Err("map_allocated_pages_to(): page count must equal frame count");
    }
    let page_table = crate::arch::page_table::page_table().lock();
    for (page, frame) in pages
        .deref()
        .clone()
        .into_iter()
        .zip(frames.deref().clone().into_iter())
    {
        page_table.map(
            page.start_address().value(),
            frame.start_address().value(),
            attr,
        );
    }
    Ok(MappedRegion {
        pages,
        frames,
        attribute: attr,
    })
}

use crate::libs::traits::Address;

pub fn virt_to_phys(virtual_address: &VAddr) -> PAddr {
    if virtual_address.is_kernel_address() {
        PAddr::new_canonical(virtual_address.value().kva2pa())
    } else {
        virtual_to_physical(virtual_address)
    }
}

// Do the transfer from user virtual address to physical address.
pub fn virtual_to_physical(virtual_address: &VAddr) -> PAddr {
    let page_table = page_table().lock();
    let entry = page_table.lookup_page(virtual_address.value()).unwrap();
    let paddr = PAddr::new_canonical(entry.pa() | virtual_address.page_offset());
    // debug!("virtual {} to physical {}", virtual_address, paddr);
    paddr
}
