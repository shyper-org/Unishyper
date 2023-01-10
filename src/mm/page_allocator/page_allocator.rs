use core::fmt;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};

use spin::Mutex;
use intrusive_collections::Bound;

use crate::arch::{PAGE_SIZE, MAX_VIRTUAL_ADDRESS, MAX_USER_VIRTUAL_ADDRESS};
use crate::mm::address::VAddr;
use crate::mm::page_allocator::page::Page;
use crate::mm::page_allocator::page_range::PageRange;
use crate::util::static_array_rb_tree::{StaticArrayRBTree, Inner, ValueRefMut, Wrapper};

const MIN_PAGE: Page = Page::containing_address(VAddr::zero());
const MAX_PAGE: Page = Page::containing_address(VAddr::new_canonical(MAX_VIRTUAL_ADDRESS));

static PAGES_UPPER_BOUND: Page =
    Page::containing_address(VAddr::new_canonical(MAX_USER_VIRTUAL_ADDRESS));

/// The single, system-wide list of free chunks of virtual memory pages.
static FREE_PAGE_LIST: Mutex<StaticArrayRBTree<Chunk>> = Mutex::new(StaticArrayRBTree::empty());

/// Initialize the page allocator.
///
/// # Arguments
/// * `end_vaddr_of_low_designated_region`: the `VirtualAddress` that marks the end of the
///   lower designated region, which should be the ending address of the initial kernel image
///   (a lower-half identity address).
///
/// The page allocator will only allocate addresses lower than `end_vaddr_of_low_designated_region`
/// if specifically requested.
/// General allocation requests for any virtual address will not use any address lower than that,
/// unless the rest of the entire virtual address space is already in use.
///
pub fn init() -> Result<(), &'static str> {
    let mut initial_free_chunks: [Option<Chunk>; 32] = Default::default();
    initial_free_chunks[0] = Some(Chunk {
        pages: PageRange::new(
            Page::containing_address(VAddr::zero()),
            PAGES_UPPER_BOUND - 1,
        ),
    });

    *FREE_PAGE_LIST.lock() = StaticArrayRBTree::new(initial_free_chunks);
    convert_to_heap_allocated();
    Ok(())
}

/// A range of contiguous pages.
///
/// # Ordering and Equality
///
/// `Chunk` implements the `Ord` trait, and its total ordering is ONLY based on
/// its **starting** `Page`. This is useful so we can store `Chunk`s in a sorted collection.
///
/// Similarly, `Chunk` implements equality traits, `Eq` and `PartialEq`,
/// both of which are also based ONLY on the **starting** `Page` of the `Chunk`.
/// Thus, comparing two `Chunk`s with the `==` or `!=` operators may not work as expected.
/// since it ignores their actual range of pages.
#[derive(Debug, Clone, Eq)]
struct Chunk {
    /// The Pages covered by this chunk, an inclusive range.
    pages: PageRange,
}
impl Chunk {
    fn as_allocated_pages(&self) -> AllocatedPages {
        AllocatedPages {
            pages: self.pages.clone(),
        }
    }

    /// Returns a new `Chunk` with an empty range of pages.
    fn empty() -> Chunk {
        Chunk {
            pages: PageRange::empty(),
        }
    }
}
impl Deref for Chunk {
    type Target = PageRange;
    fn deref(&self) -> &PageRange {
        &self.pages
    }
}
impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pages.start().cmp(other.pages.start())
    }
}
impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.pages.start() == other.pages.start()
    }
}
impl Borrow<Page> for &'_ Chunk {
    fn borrow(&self) -> &Page {
        self.pages.start()
    }
}

/// Represents a range of allocated `VirtualAddress`es, specified in `Page`s.
///
/// These pages are not initially mapped to any physical memory frames, you must do that separately
/// in order to actually use their memory; see the `MappedPages` type for more.
///
/// This object represents ownership of the allocated virtual pages;
/// if this object falls out of scope, its allocated pages will be auto-deallocated upon drop.
pub struct AllocatedPages {
    pages: PageRange,
}

// AllocatedPages must not be Cloneable, and it must not expose its inner pages as mutable.
assert_not_impl_any!(AllocatedPages: DerefMut, Clone);

impl Deref for AllocatedPages {
    type Target = PageRange;
    fn deref(&self) -> &PageRange {
        &self.pages
    }
}

impl fmt::Debug for AllocatedPages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AllocatedPages({:?})", self.pages)
    }
}

#[allow(unused)]
impl AllocatedPages {
    /// Returns an empty AllocatedPages object that performs no page allocation.
    /// Can be used as a placeholder, but will not permit any real usage.
    pub const fn empty() -> AllocatedPages {
        AllocatedPages {
            pages: PageRange::empty(),
        }
    }

    /// Merges the given `AllocatedPages` object `ap` into this `AllocatedPages` object (`self`).
    /// This is just for convenience and usability purposes, it performs no allocation or remapping.
    ///
    /// The `ap` must be virtually contiguous and come immediately after `self`,
    /// that is, `self.end` must equal `ap.start`.
    /// If this condition is met, `self` is modified and `Ok(())` is returned,
    /// otherwise `Err(ap)` is returned.
    pub fn merge(&mut self, ap: AllocatedPages) -> Result<(), AllocatedPages> {
        // make sure the pages are contiguous
        if *ap.start() != (*self.end() + 1) {
            return Err(ap);
        }
        self.pages = PageRange::new(*self.start(), *ap.end());
        // ensure the now-merged AllocatedPages doesn't run its drop handler and free its pages.
        core::mem::forget(ap);
        Ok(())
    }

    /// Splits this `AllocatedPages` into two separate `AllocatedPages` objects:
    /// * `[beginning : at_page - 1]`
    /// * `[at_page : end]`
    ///
    /// This function follows the behavior of [`core::slice::split_at()`],
    /// thus, either one of the returned `AllocatedPages` objects may be empty.
    /// * If `at_page == self.start`, the first returned `AllocatedPages` object will be empty.
    /// * If `at_page == self.end + 1`, the second returned `AllocatedPages` object will be empty.
    ///
    /// Returns an `Err` containing this `AllocatedPages` if `at_page` is otherwise out of bounds.
    pub fn split(self, at_page: Page) -> Result<(AllocatedPages, AllocatedPages), AllocatedPages> {
        let end_of_first = at_page - 1;

        let (first, second) = if at_page == *self.start() && at_page <= *self.end() {
            let first = PageRange::empty();
            let second = PageRange::new(at_page, *self.end());
            (first, second)
        } else if at_page == (*self.end() + 1) && end_of_first >= *self.start() {
            let first = PageRange::new(*self.start(), *self.end());
            let second = PageRange::empty();
            (first, second)
        } else if at_page > *self.start() && end_of_first <= *self.end() {
            let first = PageRange::new(*self.start(), end_of_first);
            let second = PageRange::new(at_page, *self.end());
            (first, second)
        } else {
            return Err(self);
        };

        // ensure the original AllocatedPages doesn't run its drop handler and free its pages.
        core::mem::forget(self);
        Ok((
            AllocatedPages { pages: first },
            AllocatedPages { pages: second },
        ))
    }
}

impl Drop for AllocatedPages {
    fn drop(&mut self) {
        if self.size_in_pages() == 0 {
            return;
        }
        trace!("page_allocator: deallocating {:?}", self);

        // Simply add the newly-deallocated chunk to the free pages list.
        let mut locked_list = FREE_PAGE_LIST.lock();
        let res = locked_list.insert(Chunk {
            pages: self.pages.clone(),
        });
        match res {
            Ok(_inserted_free_chunk) => return,
            Err(c) => error!(
                "BUG: couldn't insert deallocated chunk {:?} into free page list",
                c
            ),
        }

        // Here, we could optionally use above `_inserted_free_chunk` to merge the adjacent (contiguous) chunks
        // before or after the newly-inserted free chunk.
        // However, there's no *need* to do so until we actually run out of address space or until
        // a requested address is in a chunk that needs to be merged.
        // Thus, for performance, we save that for those future situations.
    }
}

/// A series of pending actions related to page allocator bookkeeping,
/// which may result in heap allocation.
///
/// The actions are triggered upon dropping this struct.
/// This struct can be returned from the `allocate_pages()` family of functions
/// in order to allow the caller to precisely control when those actions
/// that may result in heap allocation should occur.
/// Such actions include adding chunks to lists of free pages or pages in use.
///
/// The vast majority of use cases don't  care about such precise control,
/// so you can simply drop this struct at any time or ignore it
/// with a `let _ = ...` binding to instantly drop it.
pub struct DeferredAllocAction {
    /// A reference to the list into which we will insert the free `Chunk`s.
    // free_list: &'list Mutex<StaticArrayRBTree<Chunk>>,
    /// A free chunk that needs to be added back to the free list.
    free1: Chunk,
    /// Another free chunk that needs to be added back to the free list.
    free2: Chunk,
}
impl DeferredAllocAction {
    fn new<F1, F2>(free1: F1, free2: F2) -> DeferredAllocAction
    where
        F1: Into<Option<Chunk>>,
        F2: Into<Option<Chunk>>,
    {
        // let free_list = &FREE_PAGE_LIST;
        let free1 = free1.into().unwrap_or(Chunk::empty());
        let free2 = free2.into().unwrap_or(Chunk::empty());
        DeferredAllocAction {
            // free_list,
            free1,
            free2,
        }
    }
}

#[allow(unused)]
/// Possible allocation errors.
enum AllocationError {
    /// The requested address was not free: it was already allocated, or is outside the range of this allocator.
    AddressNotFree(Page, usize),
    /// The address space was full, or there was not a large-enough chunk
    /// or enough remaining chunks that could satisfy the requested allocation size.
    OutOfAddressSpace(usize),
    /// The allocator has not yet been initialized.
    NotInitialized,
}

impl From<AllocationError> for &'static str {
    fn from(alloc_err: AllocationError) -> &'static str {
        match alloc_err {
            AllocationError::AddressNotFree(..) => {
                "address was in use or outside of this allocator's range"
            }
            AllocationError::OutOfAddressSpace(..) => "out of address space",
            AllocationError::NotInitialized => "the allocator has not yet been initialized",
        }
    }
}

/// Searches the given `list` for the chunk that contains the range of pages from
/// `requested_page` to `requested_page + num_pages`.
fn find_specific_chunk(
    list: &mut StaticArrayRBTree<Chunk>,
    requested_page: Page,
    num_pages: usize,
) -> Result<(AllocatedPages, DeferredAllocAction), AllocationError> {
    // The end page is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
    let requested_end_page = requested_page + (num_pages - 1);

    match &mut list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    if requested_page >= *chunk.start() && requested_end_page <= *chunk.end() {
                        // Here: `chunk` was big enough and did contain the requested address.
                        return adjust_chosen_chunk(
                            requested_page,
                            num_pages,
                            &chunk.clone(),
                            ValueRefMut::Array(elem),
                        );
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            let mut cursor_mut = tree.upper_bound_mut(Bound::Included(&requested_page));
            if let Some(chunk) = cursor_mut.get().map(|w| w.deref()) {
                if requested_page >= *chunk.start() {
                    if requested_end_page <= *chunk.end() {
                        return adjust_chosen_chunk(
                            requested_page,
                            num_pages,
                            &chunk.clone(),
                            ValueRefMut::RBTree(cursor_mut),
                        );
                    } else {
                        // Here, we've found a chunk that includes the requested start page, but it's too small
                        // to cover the number of requested pages.
                        // Thus, we attempt to merge this chunk with the next contiguous chunk(s) to create one single larger chunk.
                        let chunk = chunk.clone(); // ends the above borrow on `cursor_mut`
                        let mut new_end_page = *chunk.end();
                        cursor_mut.move_next();
                        while let Some(next_chunk) = cursor_mut.get().map(|w| w.deref()) {
                            if *next_chunk.start() - 1 == new_end_page {
                                new_end_page = *next_chunk.end();
                                cursor_mut.remove().expect(
                                    "BUG: page_allocator failed to merge contiguous chunks.",
                                );
                            // The above call to `cursor_mut.remove()` advances the cursor to the next chunk.
                            } else {
                                break; // the next chunk wasn't contiguous, so stop iterating.
                            }
                        }

                        if new_end_page > *chunk.end() {
                            cursor_mut.move_prev(); // move the cursor back to the original chunk
                            let _removed_chunk = cursor_mut.replace_with(Wrapper::new_link(Chunk { pages: PageRange::new(*chunk.start(), new_end_page) }))
								.expect("BUG: page_allocator failed to replace the current chunk while merging contiguous chunks.");
                            return adjust_chosen_chunk(
                                requested_page,
                                num_pages,
                                &chunk,
                                ValueRefMut::RBTree(cursor_mut),
                            );
                        }
                    }
                }
            }
        }
    }

    Err(AllocationError::AddressNotFree(requested_page, num_pages))
}

/// Searches the given `list` for any chunk large enough to hold at least `num_pages`
/// and the start address satisfied the requirement of alignment.
fn find_alignment_chunk<'list>(
    list: &'list mut StaticArrayRBTree<Chunk>,
    alignment: usize,
    num_pages: usize,
) -> Result<(AllocatedPages, DeferredAllocAction), AllocationError> {
    // trace!("find alignment chunk");
    // During the first pass, we ignore designated regions.
    match list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    // Skip chunks that are too-small or in the designated regions.
                    if chunk.size_in_pages() < num_pages {
                        continue;
                    } else {
                        let start = *chunk.start();
                        let start_addr =
                            crate::util::round_up(start.start_address().value(), alignment);
                        let start_page = Page::containing_address(VAddr::new_canonical(start_addr));
                        let requested_end_page = start_page + (num_pages - 1);
                        if requested_end_page <= *chunk.end() {
                            return adjust_chosen_chunk(
                                *chunk.start(),
                                num_pages,
                                &chunk.clone(),
                                ValueRefMut::Array(elem),
                            );
                        }
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            // NOTE: if RBTree had a `range_mut()` method, we could simply do the following:
            // ```
            // let eligible_chunks = tree.range(
            // 	Bound::Excluded(&DESIGNATED_PAGES_LOW_END),
            // 	Bound::Excluded(&DESIGNATED_PAGES_HIGH_START)
            // );
            // for c in eligible_chunks { ... }
            // ```
            //
            // However, RBTree doesn't have a `range_mut()` method, so we use cursors for manual iteration.
            //
            // Because we allocate new pages by peeling them off from the beginning part of a chunk,
            // it's MUCH faster to start the search for free pages from higher addresses moving down.
            // This results in an O(1) allocation time in the general case, until all address ranges are already in use.
            // let mut cursor = tree.cursor_mut();
            let mut cursor = tree.upper_bound_mut(Bound::Excluded(&PAGES_UPPER_BOUND));
            while let Some(chunk) = cursor.get().map(|w| w.deref()) {
                if num_pages < chunk.size_in_pages() {
                    let start = *chunk.start();
                    let start_addr =
                        crate::util::round_up(start.start_address().value(), alignment);
                    let start_page = Page::containing_address(VAddr::new_canonical(start_addr));
                    let requested_end_page = start_page + (num_pages - 1);
                    if requested_end_page <= *chunk.end() {
                        return adjust_chosen_chunk(
                            start_page,
                            num_pages,
                            &chunk.clone(),
                            ValueRefMut::RBTree(cursor),
                        );
                    }
                }
                cursor.move_prev();
            }
        }
    }
    warn!("find alignment chunk, AllocationError::OutOfAddressSpace");
    Err(AllocationError::OutOfAddressSpace(num_pages))
}

/// Searches the given `list` for any chunk large enough to hold at least `num_pages`.
fn find_any_chunk<'list>(
    list: &'list mut StaticArrayRBTree<Chunk>,
    num_pages: usize,
) -> Result<(AllocatedPages, DeferredAllocAction), AllocationError> {
    // trace!("find any chunk");
    // During the first pass, we ignore designated regions.
    match list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    // Skip chunks that are too-small or in the designated regions.
                    if chunk.size_in_pages() < num_pages {
                        continue;
                    } else {
                        return adjust_chosen_chunk(
                            *chunk.start(),
                            num_pages,
                            &chunk.clone(),
                            ValueRefMut::Array(elem),
                        );
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            // NOTE: if RBTree had a `range_mut()` method, we could simply do the following:
            // ```
            // let eligible_chunks = tree.range(
            // 	Bound::Excluded(&DESIGNATED_PAGES_LOW_END),
            // 	Bound::Excluded(&DESIGNATED_PAGES_HIGH_START)
            // );
            // for c in eligible_chunks { ... }
            // ```
            //
            // However, RBTree doesn't have a `range_mut()` method, so we use cursors for manual iteration.
            //
            // Because we allocate new pages by peeling them off from the beginning part of a chunk,
            // it's MUCH faster to start the search for free pages from higher addresses moving down.
            // This results in an O(1) allocation time in the general case, until all address ranges are already in use.
            // let mut cursor = tree.cursor_mut();
            let mut cursor = tree.upper_bound_mut(Bound::Excluded(&PAGES_UPPER_BOUND));
            while let Some(chunk) = cursor.get().map(|w| w.deref()) {
                if num_pages < chunk.size_in_pages() {
                    return adjust_chosen_chunk(
                        *chunk.start(),
                        num_pages,
                        &chunk.clone(),
                        ValueRefMut::RBTree(cursor),
                    );
                }
                warn!("Page allocator: unlikely scenario: had to search multiple chunks while trying to allocate {} pages at any address.", num_pages);
                cursor.move_prev();
            }
        }
    }

    Err(AllocationError::OutOfAddressSpace(num_pages))
}

/// The final part of the main allocation routine.
///
/// The given chunk is the one we've chosen to allocate from.
/// This function breaks up that chunk into multiple ones and returns an `AllocatedPages`
/// from (part of) that chunk, ranging from `start_page` to `start_page + num_pages`.
fn adjust_chosen_chunk(
    start_page: Page,
    num_pages: usize,
    chosen_chunk: &Chunk,
    mut chosen_chunk_ref: ValueRefMut<Chunk>,
) -> Result<(AllocatedPages, DeferredAllocAction), AllocationError> {
    // The new allocated chunk might start in the middle of an existing chunk,
    // so we need to break up that existing chunk into 3 possible chunks: before, newly-allocated, and after.
    //
    // Because Pages and VirtualAddresses use saturating add and subtract, we need to double-check that we're not creating
    // an overlapping duplicate Chunk at either the very minimum or the very maximum of the address space.
    let new_allocation = Chunk {
        // The end page is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
        pages: PageRange::new(start_page, start_page + (num_pages - 1)),
    };
    let before = if start_page == MIN_PAGE {
        None
    } else {
        Some(Chunk {
            pages: PageRange::new(*chosen_chunk.start(), *new_allocation.start() - 1),
        })
    };
    let after = if new_allocation.end() == &MAX_PAGE {
        None
    } else {
        Some(Chunk {
            pages: PageRange::new(*new_allocation.end() + 1, *chosen_chunk.end()),
        })
    };

    // some sanity checks -- these can be removed or disabled for better performance
    if let Some(ref b) = before {
        assert!(!new_allocation.contains(b.end()));
        assert!(!b.contains(new_allocation.start()));
    }
    if let Some(ref a) = after {
        assert!(!new_allocation.contains(a.start()));
        assert!(!a.contains(new_allocation.end()));
    }

    // Remove the chosen chunk from the free page list.
    let _removed_chunk = chosen_chunk_ref.remove();
    assert_eq!(Some(chosen_chunk), _removed_chunk.as_ref()); // sanity check

    // TODO: Re-use the allocated wrapper if possible, rather than allocate a new one entirely.
    // if let RemovedValue::RBTree(Some(wrapper_adapter)) = _removed_chunk { ... }

    Ok((
        new_allocation.as_allocated_pages(),
        DeferredAllocAction::new(before, after),
    ))
}

/// The core page allocation routine that allocates the given number of virtual pages,
/// optionally at the requested starting `VAddr`.
fn inner_allocate_pages(
    requested_vaddr: Option<VAddr>,
    requested_alignment: Option<usize>,
    num_pages: usize,
) -> Option<AllocatedPages> {
    if num_pages == 0 {
        warn!("PageAllocator: requested an allocation of 0 pages... stupid!");
        return None;
    }

    let mut locked_list = FREE_PAGE_LIST.lock();

    // debug!(
    //     "inner_allocate_pages num_pages {:?} {:?} {:?}",
    //     num_pages, requested_vaddr, requested_alignment
    // );
    // The main logic of the allocator is to find an appropriate chunk that can satisfy the allocation request.
    // An appropriate chunk satisfies the following conditions:
    // - Can fit the requested size (starting at the requested address) within the chunk.
    // - Can fit the requested alignment (starting at the virtual address of specific alignment) within the chunk.
    // - The chunk can only be within in a designated region if a specific address was requested,
    //   or all other non-designated chunks are already in use.
    match if let Some(vaddr) = requested_vaddr {
        find_specific_chunk(&mut locked_list, Page::containing_address(vaddr), num_pages)
    } else if let Some(alignment) = requested_alignment {
        find_alignment_chunk(&mut locked_list, alignment, num_pages)
    } else {
        find_any_chunk(&mut locked_list, num_pages)
    } {
        Ok((ap, action)) => {
            if action.free1.size_in_pages() > 0 {
                // trace!("DeferredAllocAction insert free1 {:?}", action.free1);
                locked_list.insert(action.free1.clone()).unwrap();
            }
            if action.free2.size_in_pages() > 0 {
                // trace!("DeferredAllocAction insert free2 {:?}", action.free2);
                locked_list.insert(action.free2.clone()).unwrap();
            }
            Some(ap)
        }
        Err(e) => {
            let err: &'static str = e.into();
            error!(
                "allocate_pages error {}\nrequested_vaddr {:?} requested_alignment {:?}",
                err, requested_vaddr, requested_alignment
            );
            None
        }
    }
}

/// but accepts a size value for the allocated pages in number of bytes instead of number of pages.
///
/// This function still allocates whole pages by rounding up the number of bytes.
fn allocate_pages_by_bytes_deferred(
    requested_vaddr: Option<VAddr>,
    num_bytes: usize,
) -> Option<AllocatedPages> {
    let actual_num_bytes = if let Some(vaddr) = requested_vaddr {
        num_bytes + (vaddr.value() % PAGE_SIZE)
    } else {
        num_bytes
    };
    let num_pages = (actual_num_bytes + PAGE_SIZE - 1) / PAGE_SIZE; // round up
    inner_allocate_pages(requested_vaddr, None, num_pages)
}

/// Allocates the given number of pages,
/// with no constraints on the starting virtual address,
/// with no constraints on the virtual address alignment.
pub fn allocate_pages(num_pages: usize) -> Option<AllocatedPages> {
    trace!("allocate_pages num_pages {:?}", num_pages);
    inner_allocate_pages(None, None, num_pages)
}

/// Allocates the given number of pages
/// with no constraints on the starting virtual address,
/// with constraints on the virtual address alignment.
#[allow(unused)]
pub fn allocate_pages_alignment(num_pages: usize, alignment: usize) -> Option<AllocatedPages> {
    trace!("allocate_pages_alignment num_pages {:?}", num_pages);

    inner_allocate_pages(None, Some(alignment), num_pages)
}

/// Allocates the given number of pages starting at (inclusive of) the page containing the given `VAddr`.
pub fn allocate_pages_at(vaddr: VAddr, num_pages: usize) -> Option<AllocatedPages> {
    inner_allocate_pages(Some(vaddr), None, num_pages)
}

/// Allocates pages with no constraints on the starting virtual address,
/// with a size given by the number of bytes.
///
/// This function still allocates whole pages by rounding up the number of bytes.
#[allow(unused)]
pub fn allocate_pages_by_bytes(num_bytes: usize) -> Option<AllocatedPages> {
    allocate_pages_by_bytes_deferred(None, num_bytes)
}

/// Converts the page allocator from using static memory (a primitive array) to dynamically-allocated memory.
///
/// Call this function once heap allocation is available.
/// Calling this multiple times is unnecessary but harmless, as it will do nothing after the first invocation.
pub fn convert_to_heap_allocated() {
    FREE_PAGE_LIST.lock().convert_to_heap_allocated();
}

/// A debugging function used to dump the full internal state of the page allocator.
pub fn dump_page_allocator_state() {
    println!("--------------- FREE PAGES LIST ---------------");
    for c in FREE_PAGE_LIST.lock().iter() {
        println!(" {:X?}", c);
    }
    println!("-----------------------------------------------");
}
