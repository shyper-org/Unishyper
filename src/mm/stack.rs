use core::ops::{Deref, DerefMut};

use crate::arch::PAGE_SIZE;
use crate::mm::page_allocator;
use crate::mm::page_allocator::AllocatedPages;
use crate::mm::paging::{MappedRegion, map_allocated_pages, EntryAttribute};
use crate::mm::address::VAddr;
use crate::mm::interface::PageTableEntryAttrTrait;

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

#[allow(unused)]
impl Stack {
    /// Returns the address just beyond the top of this stack,
    /// which is necessary for some hardware registers to use.
    ///
    /// This address is not dereferenceable, the one right below it is.
    /// To get the highest usable address in this Stack, call `top_usable()`
    pub fn top_unusable(&self) -> VAddr {
        self.region.end().start_address() + PAGE_SIZE
    }

    /// Returns the highest usable address of this Stack,
    /// which is `top_unusable() - sizeof(VirtualAddress)`
    pub fn top_usable(&self) -> VAddr {
        self.top_unusable() - core::mem::size_of::<VAddr>()
    }

    /// Returns the bottom of this stack, its lowest usable address.
    pub fn bottom(&self) -> VAddr {
        self.region.start_address()
    }
}

/// Allocates a new stack and maps it to the active page table.
///
/// This also reserves an unmapped guard page beneath the bottom of the stack
/// in order to catch stack overflows.
///
/// Returns the newly-allocated stack and a VMA to represent its mapping.
pub fn alloc_stack(size_in_pages: usize) -> Option<Stack> {
    // Allocate enough pages for an additional guard page.
    let pages = page_allocator::allocate_pages(size_in_pages + 1)?;
    inner_alloc_stack(pages)
}

/// The inner implementation of stack allocation.
///
/// `pages` is the combined `AllocatedPages` object that holds
/// the guard page followed by the actual stack pages to be mapped.
fn inner_alloc_stack(pages: AllocatedPages) -> Option<Stack> {
    let start_of_stack_pages = *pages.start() + 1;
    let (guard_page, stack_pages) = pages.split(start_of_stack_pages).ok()?;

    let attr = EntryAttribute::user_default();

    // Map stack pages to physical frames, leave the guard page unmapped.
    let stack_region = match map_allocated_pages(stack_pages, attr) {
        Ok(stack_region) => stack_region,
        Err(e) => {
            error!(
                "alloc_stack(): couldn't map pages for the new Stack, error: {}",
                e
            );
            return None;
        }
    };
    // debug!("alloc stack stack_region {:?}", &stack_region);
    Some(Stack {
        guard_page,
        region: stack_region,
    })
}
