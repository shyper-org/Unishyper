use core::ops::{Deref, DerefMut};

use crate::mm::page_allocator;
use crate::mm::frame_allocator;
use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::page_allocator::AllocatedPages;
use crate::mm::paging::{MappedRegion, map_allocated_pages_to, EntryAttribute};
// use crate::mm::address::VAddr;
use crate::mm::interface::PageTableEntryAttrTrait;
// use crate::mm::interface::MapGranularity;
#[cfg(feature = "zone")]
use crate::mm::interface::PageTableEntryAttrZoneTrait;
use zone::ZoneId;

// static COUNT: AtomicUsize = AtomicUsize::new(1);

/// A range of mapped memory designated for use as a task's stack.
///
/// There is an unmapped guard page beneath the stack,
/// which is a standard approach to detect stack overflow.
///
/// A stack is backed by and auto-derefs into `MappedPages`.
#[derive(Debug)]
pub struct Stack {
    #[allow(unused)]
    guard_page: AllocatedPages,
    region: MappedRegion,
}
impl Deref for Stack {
    type Target = MappedRegion;
    fn deref(&self) -> &MappedRegion {
        &self.region
    }
}
impl DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut MappedRegion {
        &mut self.region
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        debug!(
            "Drop stack at region [{} to {}]",
            self.region.start_address(),
            self.region.start_address() + self.region.size_in_bytes()
        );
    }
}

/// Allocates a new stack and maps it to the active page table.
///
/// This also reserves an unmapped guard page beneath the bottom of the stack
/// in order to catch stack overflows.
///
/// |-----------------------------------|
/// |-                                 -|
/// |-           stack range           -|
/// |-             mapped              -|
/// |-                                 -|
/// |-----------------------------------|
/// |----------- guard page ------------|
/// |-----------------------------------|
///
/// Returns the newly-allocated stack and a VMA to represent its mapping.
pub fn alloc_stack(size_in_pages: usize, zone_id: ZoneId) -> Option<Stack> {
    // Get suggested VAddr for stack.
    // let pages: AllocatedPages;
    // loop {
    //     // Search for appropriate stack region.
    //     let count = COUNT.fetch_add(2, Ordering::AcqRel);
    //     let stack_addr =
    //         VAddr::new_canonical(count * STACK_SIZE + crate::arch::MIN_USER_VIRTUAL_ADDRESS);
    //     trace!("alloc stack loop: count {} saddr {}", count, stack_addr);
    //     // Allocate enough pages for an additional guard page.
    //     if let Some(aps) =
    //         page_allocator::allocate_pages_at(stack_addr - PAGE_SIZE, size_in_pages + 1)
    //     {
    //         pages = aps;
    //         trace!("alloc stack loop: get count {} saddr {}", count, stack_addr);
    //         break;
    //     }
    // }
    // // Get physical address for stack, no need to alloc space for guarded page.
    // let frames = frame_allocator::allocate_frames_alignment(
    //     size_in_pages,
    //     MapGranularity::Page2MB as usize,
    // )?;
    assert_eq!(size_in_pages >= 2, true);
    let pages = page_allocator::allocate_pages(size_in_pages)?;
    let frames = frame_allocator::allocate_frames(size_in_pages - 1)?;
    trace!("alloc_stack pages {:?}", &pages);
    trace!("alloc_stack frames {:?}", &frames);
    inner_alloc_stack(pages, frames, zone_id)
}

/// The inner implementation of stack allocation.
///
/// `pages` is the combined `AllocatedPages` object that holds
///  the guard page followed by the actual stack pages to be mapped.
#[allow(unused_mut)]
fn inner_alloc_stack(
    pages: AllocatedPages,
    frames: AllocatedFrames,
    zone_id: ZoneId,
) -> Option<Stack> {
    // Split the guard page.
    let start_of_stack_pages = *pages.start() + 1;
    let (guard_page, stack_pages) = pages.split(start_of_stack_pages).ok()?;

    let mut attr = EntryAttribute::user_default();

    #[cfg(feature = "zone")]
    attr.set_zone(zone_id);

    // Map stack pages to physical frames, leave the guard page unmapped.
    let stack_region = match map_allocated_pages_to(stack_pages, frames, attr) {
        Ok(stack_region) => stack_region,
        Err(e) => {
            error!(
                "alloc_stack(): couldn't map pages for the new Stack, error: {}",
                e
            );
            return None;
        }
    };
    // trace!("guard_page {:?}", &stack_pages);
    // trace!("stack_pages {:?}", &stack_pages);
    debug!(
        "stack_region {:#?}\n mapped success with zone_id {}",
        &stack_region, zone_id
    );
    Some(Stack {
        guard_page,
        region: stack_region,
    })
}
