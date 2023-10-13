use core::fmt;
use core::ops::Deref;

use crate::arch::PAGE_SHIFT;
use crate::arch::page_table::{page_table, PAGE_TABLE_L2_SHIFT};
use crate::mm::interface::{PageTableEntryAttrTrait, PageTableTrait, MapGranularity};

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
        let mut page_table = crate::arch::page_table::page_table().lock();

        if self.attribute().block() {
            // Unmap by 2mb.
            let step = 1 << (PAGE_TABLE_L2_SHIFT - PAGE_SHIFT);
            for page in self.pages.deref().clone().into_iter().step_by(step) {
                page_table.unmap_2mb(page.start_address().value());
            }
        } else {
            // Unmap by 4kb.
            for page in self.pages.deref().clone().into_iter() {
                page_table.unmap(page.start_address().value());
            }
        }
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        trace!("Drop Mapped Region at {}", self.start_address());
        self.unmap();
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
    trace!(
        "map_allocated_pages(): {} pages:{} frames:{} attr: {:?}",
        pages.size_in_pages(),
        pages.start().start_address(),
        frames.start().start_address(),
        attr
    );

    let mut page_table = crate::arch::page_table::page_table().lock();
    for (page, frame) in pages
        .deref()
        .clone()
        .into_iter()
        .zip(frames.deref().clone().into_iter())
    {
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
pub fn map_allocated_pages_to(
    pages: AllocatedPages,
    frames: AllocatedFrames,
    attr: EntryAttribute,
) -> Result<MappedRegion, &'static str> {
    // Judge if pages and frames and be mapped.
    let pages_count = pages.size_in_pages();
    let frames_count = frames.size_in_frames();
    if pages_count != frames_count {
        error!(
            "map_allocated_pages_to(): pages {:?} count {} must equal frames {:?} count {}!",
            pages, pages_count, frames, frames_count
        );
        return Err("map_allocated_pages_to(): page count must equal frame count");
    }
    // Get global page table.
    let mut page_table = crate::arch::page_table::page_table().lock();

    // Judge if can be map as 2MB.
    if pages.size_in_bytes() % MapGranularity::Page2MB as usize == 0 {
        let attr = attr.set_block();
        let step = 1 << (PAGE_TABLE_L2_SHIFT - PAGE_SHIFT);
        for (page, frame) in pages
            .deref()
            .clone()
            .into_iter()
            .zip(frames.deref().clone().into_iter())
            .step_by(step)
        {
            match page_table.map_2mb(
                page.start_address().value(),
                frame.start_address().value(),
                attr,
            ) {
                Ok(()) => continue,
                Err(_) => {
                    return Err("page table map_2mb error");
                }
            }
        }
        Ok(MappedRegion {
            pages,
            frames,
            attribute: attr,
        })
    } else {
        for (page, frame) in pages
            .deref()
            .clone()
            .into_iter()
            .zip(frames.deref().clone().into_iter())
        {
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
}

use crate::libs::traits::Address;

pub fn virt_to_phys(virtual_address: &VAddr) -> PAddr {
    if virtual_address.is_kernel_address() {
        PAddr::new_canonical(virtual_address.value().kva2pa())
    } else {
        match virtual_to_physical(virtual_address) {
            Some(paddr) => paddr,
            None => panic!("{} not mapped", virtual_address),
        }
    }
}

// Do the transfer from user virtual address to physical address.
// Need to check the mapping granularity.
pub fn virtual_to_physical(virtual_address: &VAddr) -> Option<PAddr> {
    let page_table = page_table().lock();
    let (entry, granularity) = match page_table.lookup_entry(virtual_address.value()) {
        Some((entry, granularity)) => (entry, granularity),
        None => {
            warn!("virtual_to_physical: {} is not mapped", virtual_address);
            return None;
        }
    };
    let paddr = match granularity {
        MapGranularity::Page4KB => PAddr::new_canonical(entry.pa() | virtual_address.page_offset()),
        MapGranularity::Page2MB => {
            PAddr::new_canonical(entry.pa() | virtual_address.page_offset_2mb())
        }
        MapGranularity::Page1GB => unimplemented!("mapped by 1GB currently not supported"),
    };
    Some(paddr)
}

#[allow(unused)]
pub fn map_device_memory_range(device_addr: usize, mem_size: usize) -> VAddr {
    let attr = EntryAttribute::kernel_device();
    let mut physical_address = PAddr::new_canonical(device_addr);
    use crate::mm::page_allocator;
    let pages = match page_allocator::allocate_pages_by_bytes(mem_size) {
        Some(pages) => pages,
        None => panic!("failed to allocate pages for PCI bar"),
    };
    let start_addr = pages.start_address();

    let mut page_table = crate::arch::page_table::page_table().lock();
    for page in pages.deref().clone().into_iter() {
        match page_table.map(page.start_address().value(), physical_address.value(), attr) {
            Ok(()) => {
                trace!("map paddr physical_address {} success", physical_address);
                physical_address += crate::arch::PAGE_SIZE;
            }
            Err(_) => {
                panic!("map_device_memory_range failed on {}", page.start_address());
            }
        }
    }
    // for page in pages.deref().clone().into_iter() {
    // page_table.dump_entry(pages.start_address().value());
    // }
    core::mem::forget(pages);

    start_addr
}
