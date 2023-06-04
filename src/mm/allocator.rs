use alloc::collections::BTreeMap;
use spin::Mutex;

use crate::arch::PAGE_SIZE;
use crate::libs::string::memset;
use crate::libs::thread::current_thread;
use crate::libs::traits::Address;
use crate::mm::page_allocator;
use crate::mm::frame_allocator;
use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::paging::{map_allocated_pages, EntryAttribute, MappedRegion};
use crate::mm::address::VAddr;
use crate::mm::interface::PageTableEntryAttrTrait;

static GLOBAL_MM_MAP: Mutex<BTreeMap<VAddr, AllocatedFrames>> = Mutex::new(BTreeMap::new());

/// Special function for kernel page alloc.
/// Just need to alloc frames, because kernel can access it through page table store in TTBR1_EL1.
#[allow(unused)]
pub fn kallocate(size: usize) -> Option<VAddr> {
    assert!(size > 0);
    assert_eq!(
        size % PAGE_SIZE,
        0,
        "Size {:#X} is not a multiple of {:#X}",
        size,
        size % PAGE_SIZE,
    );
    let num_frames = size / PAGE_SIZE;
    let frames = match frame_allocator::allocate_frames(num_frames) {
        Some(frames) => frames,
        None => {
            warn!(
                "kallocate(): Failed to allocate frames of size {:x}, OOM",
                size
            );
            return None;
        }
    };
    let addr = frames.start_address();
    let kaddr = VAddr::new_canonical(addr.value().pa2kva());
    // Zero allocated memory space.
    unsafe {
        memset(kaddr.value() as *mut u8, 0, size);
    }
    // debug!("kernel allocate size {:x} {} to => {}", size, addr, kaddr);
    GLOBAL_MM_MAP.lock().insert(kaddr, frames);
    Some(kaddr)
}

pub fn allocate_region(size: usize) -> Result<MappedRegion, &'static str> {
    trace!("user allocate region size {:#x}", size);
    assert!(size > 0);
    assert_eq!(
        size % PAGE_SIZE,
        0,
        "Size {:#X} is not a multiple of {:#X}",
        size,
        size % PAGE_SIZE,
    );
    let size_in_pages = size / PAGE_SIZE;
    let pages = match page_allocator::allocate_pages(size_in_pages) {
        Some(pages) => pages,
        None => {
            return Err("allocate_region(): Failed to allocate");
        }
    };
    let attr = EntryAttribute::user_default();
    map_allocated_pages(pages, attr)
}

pub fn allocate(size: usize) -> Option<VAddr> {
    debug!("user allocate size {:x}", size);
    assert!(size > 0);
    assert_eq!(
        size % PAGE_SIZE,
        0,
        "Size {:#X} is not a multiple of {:#X}",
        size,
        size % PAGE_SIZE,
    );
    let size_in_pages = size / PAGE_SIZE;
    let pages = match page_allocator::allocate_pages(size_in_pages) {
        Some(pages) => pages,
        None => {
            warn!("allocate(): Failed to allocate mem size {:x}, OOM", size);
            return None;
        }
    };

    let attr = EntryAttribute::user_default();

    let region = match map_allocated_pages(pages, attr) {
        Ok(region) => region,
        Err(e) => {
            warn!(
                "allocate(): Couldn't map pages for the new region, error: {}",
                e
            );
            return None;
        }
    };
    // debug!("allocate region start 0x{:x} size 0x{:x}", region.va(), region.size());

    let addr = region.start_address();

    match current_thread() {
        Ok(t) => {
            debug!(
                "allocate(): thread {} alloc size 0x{:x} pages_num {} region start 0x{:x} size 0x{:x}",
                t.tid(),
                size,
                size / PAGE_SIZE,
                addr.value(),
                region.size_in_bytes()
            );
            t.add_mem_region(addr, region);
        }
        Err(_) => {
            warn!("allocate(): BUG,Illegal dellocate {}, only kernel virtual address can be allocated without current thread", addr);
        }
    };

    Some(addr)
}

pub fn deallocate(address: VAddr) {
    // Handle kernel virtual address deallocation.
    if address.is_kernel_address() {
        match GLOBAL_MM_MAP.lock().remove(&address) {
            Some(_) => {
                trace!("deallocate(): drop kernel virtual address {}", address);
            }
            None => {
                warn!(
                    "deallocate(): BUG, Kernel virtual address {} unexist",
                    address
                );
            }
        }
    }

    // Handle user virtual address deallocation.
    match current_thread() {
        Ok(t) => {
            t.free_mem_region(address);
            trace!(
                "deallocate(): Thread [{}] deallocate region addr start 0x{:x}",
                t.tid(),
                address.value()
            );
        }
        Err(_) => {
            warn!("deallocate(): BUG, no current thread!");
        }
    };
}
