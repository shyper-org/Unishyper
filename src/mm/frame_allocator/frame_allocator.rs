//! Provides an allocator for physical memory frames.
//! The minimum unit of allocation is a single frame.

use core::fmt;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::ops::{Deref, DerefMut};

use spin::Mutex;
use intrusive_collections::Bound;

use crate::mm::address::PAddr;
use crate::mm::frame_allocator::frame::Frame;
use crate::mm::frame_allocator::frame_range::FrameRange;
use crate::util::static_array_rb_tree::{StaticArrayRBTree, Inner, ValueRefMut};

#[allow(unused)]
const FRAME_SIZE: usize = crate::arch::PAGE_SIZE;
const MIN_FRAME: Frame = Frame::containing_address(PAddr::zero());
const MAX_FRAME: Frame = Frame::containing_address(PAddr::new_canonical(usize::MAX));

/// The single, system-wide list of free physical memory frames available for general usage.
static FREE_GENERAL_FRAMES_LIST: Mutex<StaticArrayRBTree<Chunk>> =
    Mutex::new(StaticArrayRBTree::empty());

/// Initialize the frame allocator with the given list of available and reserved physical memory regions.
///
/// Any regions in either of the lists may overlap, this is checked for and handled properly.
/// Reserved regions take priority -- if a reserved region partially or fully overlaps any part of a free region,
/// that portion will be considered reserved, not free.
///
/// The iterator (`R`) over reserved physical memory regions must be cloneable,
/// as this runs before heap allocation is available, and we may need to iterate over it multiple times.
pub fn init() -> Result<(), &'static str> {
    if FREE_GENERAL_FRAMES_LIST.lock().len() != 0 {
        return Err("BUG: Frame allocator was already initialized, cannot be initialized twice.");
    }

    let mut free_list: [Option<Chunk>; 32] = Default::default();

    for (idx, frame_range) in crate::mm::config::paged_ranges().iter().enumerate() {
        free_list[idx] = Some(Chunk {
            frames: FrameRange::from_phys_addr(
                PAddr::new(frame_range.start).unwrap(),
                frame_range.end - frame_range.start,
            ),
        });
    }

    *FREE_GENERAL_FRAMES_LIST.lock() = StaticArrayRBTree::new(free_list.clone());
    convert_to_heap_allocated();
    Ok(())
}

/// A range of contiguous frames.
///
/// # Ordering and Equality
///
/// `Chunk` implements the `Ord` trait, and its total ordering is ONLY based on
/// its **starting** `Frame`. This is useful so we can store `Chunk`s in a sorted collection.
///
/// Similarly, `Chunk` implements equality traits, `Eq` and `PartialEq`,
/// both of which are also based ONLY on the **starting** `Frame` of the `Chunk`.
/// Thus, comparing two `Chunk`s with the `==` or `!=` operators may not work as expected.
/// since it ignores their actual range of frames.
#[derive(Clone, Eq)]
struct Chunk {
    /// The Frames covered by this chunk, an inclusive range.
    frames: FrameRange,
}
impl Chunk {
    fn as_allocated_frames(&self) -> AllocatedFrames {
        AllocatedFrames {
            frames: self.frames.clone(),
        }
    }

    /// Returns a new `Chunk` with an empty range of frames.
    fn empty() -> Chunk {
        Chunk {
            frames: FrameRange::empty(),
        }
    }
}
impl Deref for Chunk {
    type Target = FrameRange;
    fn deref(&self) -> &FrameRange {
        &self.frames
    }
}
impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.frames.start().cmp(other.frames.start())
    }
}
impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.frames.start() == other.frames.start()
    }
}
impl Borrow<Frame> for &'_ Chunk {
    fn borrow(&self) -> &Frame {
        self.frames.start()
    }
}
impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chunk [{:?}]", self.frames)
    }
}

/// Represents a range of allocated `PAddr`es, specified in `Frame`s.
///
/// These frames are not initially mapped to any physical memory frames, you must do that separately
/// in order to actually use their memory; see the `MappedFrames` type for more.
///
/// This object represents ownership of the allocated physical frames;
/// if this object falls out of scope, its allocated frames will be auto-deallocated upon drop.
pub struct AllocatedFrames {
    frames: FrameRange,
}

// AllocatedFrames must not be Cloneable, and it must not expose its inner frames as mutable.
assert_not_impl_any!(AllocatedFrames: DerefMut, Clone);

impl Deref for AllocatedFrames {
    type Target = FrameRange;
    fn deref(&self) -> &FrameRange {
        &self.frames
    }
}
impl fmt::Debug for AllocatedFrames {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AllocatedFrames({:?})", self.frames)
    }
}

#[allow(unused)]
impl AllocatedFrames {
    /// Returns an empty AllocatedFrames object that performs no frame allocation.
    /// Can be used as a placeholder, but will not permit any real usage.
    pub const fn empty() -> AllocatedFrames {
        AllocatedFrames {
            frames: FrameRange::empty(),
        }
    }

    /// Merges the given `AllocatedFrames` object `other` into this `AllocatedFrames` object (`self`).
    /// This is just for convenience and usability purposes, it performs no allocation or remapping.
    ///
    /// The given `other` must be physically contiguous with `self`, i.e., come immediately before or after `self`.
    /// That is, either `self.start == other.end + 1` or `self.end + 1 == other.start` must be true.
    ///
    /// If either of those conditions are met, `self` is modified and `Ok(())` is returned,
    /// otherwise `Err(other)` is returned.
    pub fn merge(&mut self, other: AllocatedFrames) -> Result<(), AllocatedFrames> {
        if *self.start() == *other.end() + 1 {
            // `other` comes contiguously before `self`
            self.frames = FrameRange::new(*other.start(), *self.end());
        } else if *self.end() + 1 == *other.start() {
            // `self` comes contiguously before `other`
            self.frames = FrameRange::new(*self.start(), *other.end());
        } else {
            // non-contiguous
            return Err(other);
        }

        // ensure the now-merged AllocatedFrames doesn't run its drop handler and free its frames.
        core::mem::forget(other);
        Ok(())
    }

    /// Splits this `AllocatedFrames` into two separate `AllocatedFrames` objects:
    /// * `[beginning : at_frame - 1]`
    /// * `[at_frame : end]`
    ///
    /// This function follows the behavior of [`core::slice::split_at()`],
    /// thus, either one of the returned `AllocatedFrames` objects may be empty.
    /// * If `at_frame == self.start`, the first returned `AllocatedFrames` object will be empty.
    /// * If `at_frame == self.end + 1`, the second returned `AllocatedFrames` object will be empty.
    ///
    /// Returns an `Err` containing this `AllocatedFrames` if `at_frame` is otherwise out of bounds.
    pub fn split(
        self,
        at_frame: Frame,
    ) -> Result<(AllocatedFrames, AllocatedFrames), AllocatedFrames> {
        let end_of_first = at_frame - 1;

        let (first, second) = if at_frame == *self.start() && at_frame <= *self.end() {
            let first = FrameRange::empty();
            let second = FrameRange::new(at_frame, *self.end());
            (first, second)
        } else if at_frame == (*self.end() + 1) && end_of_first >= *self.start() {
            let first = FrameRange::new(*self.start(), *self.end());
            let second = FrameRange::empty();
            (first, second)
        } else if at_frame > *self.start() && end_of_first <= *self.end() {
            let first = FrameRange::new(*self.start(), end_of_first);
            let second = FrameRange::new(at_frame, *self.end());
            (first, second)
        } else {
            return Err(self);
        };

        // ensure the original AllocatedFrames doesn't run its drop handler and free its frames.
        core::mem::forget(self);
        Ok((
            AllocatedFrames { frames: first },
            AllocatedFrames { frames: second },
        ))
    }
}

impl Drop for AllocatedFrames {
    fn drop(&mut self) {
        if self.size_in_frames() == 0 {
            return;
        }
        trace!("frame_allocator: deallocating {:?}", self);

        // Simply add the newly-deallocated chunk to the free frames list.
        let mut locked_list = FREE_GENERAL_FRAMES_LIST.lock();
        let res = locked_list.insert(Chunk {
            frames: self.frames.clone(),
        });
        match res {
            Ok(_inserted_free_chunk) => return,
            Err(c) => error!(
                "BUG: couldn't insert deallocated chunk {:?} into free frame list",
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

/// A series of pending actions related to frame allocator bookkeeping,
/// which may result in heap allocation.
///
/// The actions are triggered upon dropping this struct.
/// This struct can be returned from the `allocate_frames()` family of functions
/// in order to allow the caller to precisely control when those actions
/// that may result in heap allocation should occur.
/// Such actions include adding chunks to lists of free frames or frames in use.
///
/// The vast majority of use cases don't care about such precise control,
/// so you can simply drop this struct at any time or ignore it
/// with a `let _ = ...` binding to instantly drop it.
pub struct DeferredAllocAction {
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
        let free1 = free1.into().unwrap_or(Chunk::empty());
        let free2 = free2.into().unwrap_or(Chunk::empty());
        DeferredAllocAction { free1, free2 }
    }
}

/// Possible allocation errors.
enum AllocationError {
    /// The requested address was not free: it was already allocated, or is outside the range of this allocator.
    AddressNotFree(Frame, usize),
    /// The address space was full, or there was not a large-enough chunk
    /// or enough remaining chunks that could satisfy the requested allocation size.
    OutOfAddressSpace(usize),
}
impl From<AllocationError> for &'static str {
    fn from(alloc_err: AllocationError) -> &'static str {
        match alloc_err {
            AllocationError::AddressNotFree(..) => {
                "address was in use or outside of this allocator's range"
            }
            AllocationError::OutOfAddressSpace(..) => "out of address space",
        }
    }
}

/// Searches the given `list` for the chunk that contains the range of frames from
/// `requested_frame` to `requested_frame + num_frames`.
fn find_specific_chunk(
    list: &mut StaticArrayRBTree<Chunk>,
    requested_frame: Frame,
    num_frames: usize,
) -> Result<(AllocatedFrames, DeferredAllocAction), AllocationError> {
    // The end frame is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
    let requested_end_frame = requested_frame + (num_frames - 1);

    match &mut list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    if requested_frame >= *chunk.start() && requested_end_frame <= *chunk.end() {
                        // Here: `chunk` was big enough and did contain the requested address.
                        return allocate_from_chosen_chunk(
                            requested_frame,
                            num_frames,
                            &chunk.clone(),
                            ValueRefMut::Array(elem),
                        );
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            let mut cursor_mut = tree.upper_bound_mut(Bound::Included(&requested_frame));
            if let Some(chunk) = cursor_mut.get().map(|w| w.deref().clone()) {
                if chunk.contains(&requested_frame) {
                    if requested_end_frame <= *chunk.end() {
                        return allocate_from_chosen_chunk(
                            requested_frame,
                            num_frames,
                            &chunk.clone(),
                            ValueRefMut::RBTree(cursor_mut),
                        );
                    } else {
                        // We found the chunk containing the requested address, but it was too small to cover all of the requested frames.
                        // Let's try to merge the next-highest contiguous chunk to see if those two chunks together
                        // cover enough frames to fulfill the allocation request.
                        //
                        // trace!("Frame allocator: found chunk containing requested address, but it was too small. \
                        //     Attempting to merge multiple chunks during an allocation. \
                        //     Requested address: {:?}, num_frames: {}, chunk: {:?}",
                        //     requested_frame, num_frames, chunk,
                        // );
                        let next_contiguous_chunk: Option<Chunk> = {
                            let next_cursor = cursor_mut.peek_next();
                            if let Some(next_chunk) = next_cursor.get().map(|w| w.deref()) {
                                if *chunk.end() + 1 == *next_chunk.start() {
                                    // Here: next chunk was contiguous with the original chunk.
                                    if requested_end_frame <= *next_chunk.end() {
                                        // trace!("Frame allocator: found suitably-large contiguous next {:?} after initial too-small {:?}", next_chunk, chunk);
                                        Some(next_chunk.clone())
                                    } else {
                                        todo!("Frame allocator: found chunk containing requested address, but it was too small. \
                                            Shyper does not yet support merging more than two chunks during an allocation request. \
                                            Requested address: {:?}, num_frames: {}, chunk: {:?}, next_chunk {:?}",
                                            requested_frame, num_frames, chunk, next_chunk
                                        );
                                        // None
                                    }
                                } else {
                                    trace!("Frame allocator: next {:?} was not contiguously above initial too-small {:?}", next_chunk, chunk);
                                    None
                                }
                            } else {
                                trace!("Frame allocator: couldn't get next chunk above initial too-small {:?}", chunk);
                                None
                            }
                        };
                        if let Some(mut next_chunk) = next_contiguous_chunk {
                            // We found a suitable chunk that came contiguously after the initial too-small chunk.
                            // Remove the initial chunk (since we have a cursor pointing to it already)
                            // and "merge" it into this `next_chunk`.
                            let _removed_initial_chunk = cursor_mut.remove();
                            // trace!("Frame allocator: removed suitably-large contiguous next {:?} after initial too-small {:?}", _removed_initial_chunk, chunk);
                            // Here, `cursor_mut` has been moved forward to point to the `next_chunk` now.
                            next_chunk.frames = FrameRange::new(*chunk.start(), *next_chunk.end());
                            return allocate_from_chosen_chunk(
                                requested_frame,
                                num_frames,
                                &next_chunk,
                                ValueRefMut::RBTree(cursor_mut),
                            );
                        }
                    }
                }
            }
        }
    }

    Err(AllocationError::AddressNotFree(requested_frame, num_frames))
}

/// Searches the given `list` for any chunk large enough to hold at least `num_frames`
/// and the start address satisfied the requirement of alignment.
fn find_alignment_chunk<'list>(
    list: &'list mut StaticArrayRBTree<Chunk>,
    alignment: usize,
    num_frames: usize,
) -> Result<(AllocatedFrames, DeferredAllocAction), AllocationError> {
    // During the first pass, we ignore designated regions.
    match list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    // Skip chunks that are too-small or in the designated regions.
                    if chunk.size_in_frames() < num_frames {
                        continue;
                    } else {
                        let start = *chunk.start();
                        let start_addr =
                            crate::util::round_up(start.start_address().value(), alignment);
                        let start_frame =
                            Frame::containing_address(PAddr::new_canonical(start_addr));
                        let requested_end_frame = start_frame + (num_frames - 1);
                        if requested_end_frame <= *chunk.end() {
                            return allocate_from_chosen_chunk(
                                *chunk.start(),
                                num_frames,
                                &chunk.clone(),
                                ValueRefMut::Array(elem),
                            );
                        }
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            // Because we allocate new frames by peeling them off from the beginning part of a chunk,
            // it's MUCH faster to start the search for free frames from higher addresses moving down.
            // This results in an O(1) allocation time in the general case, until all address ranges are already in use.
            // let mut cursor = tree.cursor_mut();
            let mut cursor = tree.upper_bound_mut(Bound::<&Chunk>::Unbounded);
            while let Some(chunk) = cursor.get().map(|w| w.deref()) {
                if num_frames < chunk.size_in_frames() {
                    let start = *chunk.start();
                    let start_addr =
                        crate::util::round_up(start.start_address().value(), alignment);
                    let start_frame = Frame::containing_address(PAddr::new_canonical(start_addr));
                    let requested_end_frame = start_frame + (num_frames - 1);
                    if requested_end_frame <= *chunk.end() {
                        return allocate_from_chosen_chunk(
                            start_frame,
                            num_frames,
                            &chunk.clone(),
                            ValueRefMut::RBTree(cursor),
                        );
                    }
                }
                cursor.move_prev();
            }
        }
    }

    Err(AllocationError::OutOfAddressSpace(num_frames))
}

/// Searches the given `list` for any chunk large enough to hold at least `num_frames`.
fn find_any_chunk<'list>(
    list: &'list mut StaticArrayRBTree<Chunk>,
    num_frames: usize,
) -> Result<(AllocatedFrames, DeferredAllocAction), AllocationError> {
    // During the first pass, we ignore designated regions.
    match list.0 {
        Inner::Array(ref mut arr) => {
            for elem in arr.iter_mut() {
                if let Some(chunk) = elem {
                    // Skip chunks that are too-small or in the designated regions.
                    if chunk.size_in_frames() < num_frames {
                        continue;
                    } else {
                        return allocate_from_chosen_chunk(
                            *chunk.start(),
                            num_frames,
                            &chunk.clone(),
                            ValueRefMut::Array(elem),
                        );
                    }
                }
            }
        }
        Inner::RBTree(ref mut tree) => {
            // Because we allocate new frames by peeling them off from the beginning part of a chunk,
            // it's MUCH faster to start the search for free frames from higher addresses moving down.
            // This results in an O(1) allocation time in the general case, until all address ranges are already in use.
            let mut cursor = tree.upper_bound_mut(Bound::<&Chunk>::Unbounded);
            while let Some(chunk) = cursor.get().map(|w| w.deref()) {
                if num_frames <= chunk.size_in_frames() {
                    return allocate_from_chosen_chunk(
                        *chunk.start(),
                        num_frames,
                        &chunk.clone(),
                        ValueRefMut::RBTree(cursor),
                    );
                }
                warn!(
                    "Frame allocator: inefficient scenario: had to search multiple chunks \
                    (skipping {:?}) while trying to allocate {} frames at any address.",
                    chunk, num_frames
                );
                cursor.move_prev();
            }
        }
    }

    error!(
        "frame_allocator: non-reserved chunks are all allocated (requested {} frames). \
        TODO: we could attempt to merge free chunks here.",
        num_frames
    );

    Err(AllocationError::OutOfAddressSpace(num_frames))
}

/// The final part of the main allocation routine that splits the given chosen chunk
/// into multiple smaller chunks, thereby "allocating" frames from it.
///
/// This function breaks up that chunk into multiple ones and returns an `AllocatedFrames`
/// from (part of) that chunk, ranging from `start_frame` to `start_frame + num_frames`.
fn allocate_from_chosen_chunk(
    start_frame: Frame,
    num_frames: usize,
    chosen_chunk: &Chunk,
    mut chosen_chunk_ref: ValueRefMut<Chunk>,
) -> Result<(AllocatedFrames, DeferredAllocAction), AllocationError> {
    let (new_allocation, before, after) = split_chosen_chunk(start_frame, num_frames, chosen_chunk);

    // Remove the chosen chunk from the free frame list.
    let _removed_chunk = chosen_chunk_ref.remove();

    // TODO: Re-use the allocated wrapper if possible, rather than allocate a new one entirely.
    // if let RemovedValue::RBTree(Some(wrapper_adapter)) = _removed_chunk { ... }

    Ok((
        new_allocation.as_allocated_frames(),
        DeferredAllocAction::new(before, after),
    ))
}

/// An inner function that breaks up the given chunk into multiple smaller chunks.
///
/// Returns a tuple of three chunks:
/// 1. The `Chunk` containing the requested range of frames starting at `start_frame`.
/// 2. The range of frames in the `chosen_chunk` that came before the beginning of the requested frame range.
/// 3. The range of frames in the `chosen_chunk` that came after the end of the requested frame range.
fn split_chosen_chunk(
    start_frame: Frame,
    num_frames: usize,
    chosen_chunk: &Chunk,
) -> (Chunk, Option<Chunk>, Option<Chunk>) {
    // The new allocated chunk might start in the middle of an existing chunk,
    // so we need to break up that existing chunk into 3 possible chunks: before, newly-allocated, and after.
    //
    // Because Frames and PhysicalAddresses use saturating add/subtract, we need to double-check that
    // we don't create overlapping duplicate Chunks at either the very minimum or the very maximum of the address space.
    let new_allocation = Chunk {
        // The end frame is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
        frames: FrameRange::new(start_frame, start_frame + (num_frames - 1)),
    };
    let before = if start_frame == MIN_FRAME {
        None
    } else {
        Some(Chunk {
            frames: FrameRange::new(*chosen_chunk.start(), *new_allocation.start() - 1),
        })
    };
    let after = if new_allocation.end() == &MAX_FRAME {
        None
    } else {
        Some(Chunk {
            frames: FrameRange::new(*new_allocation.end() + 1, *chosen_chunk.end()),
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

    (new_allocation, before, after)
}

/// The core frame allocation routine that allocates the given number of physical frames,
/// optionally at the requested starting `PAddr`.
///
/// This simply reserves a range of frames; it does not perform any memory mapping.
/// Thus, the memory represented by the returned `AllocatedFrames` isn't directly accessible
/// until you map virtual pages to them.
///
/// Allocation is based on a red-black tree and is thus `O(log(n))`.
/// Fragmentation isn't cleaned up until we're out of address space, but that's not really a big deal.
///
/// # Arguments
/// * `requested_paddr`: if `Some`, the returned `AllocatedFrames` will start at the `Frame`
///   containing this `PAddr`.
///   If `None`, the first available `Frame` range will be used, starting at any random physical address.
/// * `num_frames`: the number of `Frame`s to be allocated.
///
/// # Return
/// If successful, returns a tuple of two items:
/// * the frames that were allocated, and
/// * an opaque struct representing details of bookkeeping-related actions that may cause heap allocation.
///   Those actions are deferred until this returned `DeferredAllocAction` struct object is dropped,
///   allowing the caller (such as the heap implementation itself) to control when heap allocation may occur.
pub fn allocate_frames_deferred(
    requested_paddr: Option<PAddr>,
    requested_alignment: Option<usize>,
    num_frames: usize,
) -> Option<AllocatedFrames> {
    if num_frames == 0 {
        warn!("frame_allocator: requested an allocation of 0 frames... stupid!");
        return None;
    }

    let mut locked_list = FREE_GENERAL_FRAMES_LIST.lock();

    match if let Some(paddr) = requested_paddr {
        find_specific_chunk(
            &mut locked_list,
            Frame::containing_address(paddr),
            num_frames,
        )
    } else if let Some(alignment) = requested_alignment {
        find_alignment_chunk(&mut locked_list, alignment, num_frames)
    } else {
        find_any_chunk(&mut locked_list, num_frames)
    } {
        Ok((ap, action)) => {
            if action.free1.size_in_frames() > 0 {
                // trace!("DeferredAllocAction insert free1 {:?}", action.free1);
                locked_list.insert(action.free1.clone()).unwrap();
            }
            if action.free2.size_in_frames() > 0 {
                // trace!("DeferredAllocAction insert free2 {:?}", action.free2);
                locked_list.insert(action.free2.clone()).unwrap();
            }
            Some(ap)
        }
        Err(e) => {
            let err: &'static str = e.into();
            error!("allocate_frames error {}", err);
            None
        }
    }
}

/// Similar to [`allocated_frames_deferred()`](fn.allocate_frames_deferred.html),
/// but accepts a size value for the allocated frames in number of bytes instead of number of frames.
///
/// This function still allocates whole frames by rounding up the number of bytes.
#[allow(unused)]
pub fn allocate_frames_by_bytes_deferred(
    requested_paddr: Option<PAddr>,
    requested_alignment: Option<usize>,
    num_bytes: usize,
) -> Option<AllocatedFrames> {
    let actual_num_bytes = if let Some(paddr) = requested_paddr {
        num_bytes + (paddr.value() % FRAME_SIZE)
    } else {
        num_bytes
    };
    let num_frames = (actual_num_bytes + FRAME_SIZE - 1) / FRAME_SIZE; // round up
    allocate_frames_deferred(requested_paddr, None, num_frames)
}

/// Allocates the given number of frames with no constraints on the starting physical address.
pub fn allocate_frames(num_frames: usize) -> Option<AllocatedFrames> {
    trace!("allocate {} frames", num_frames);
    allocate_frames_deferred(None, None, num_frames)
}

/// Allocates the given number of frames
/// with no constraints on the starting physical address.
/// with constraints on the physical address alignment.
pub fn allocate_frames_alignment(num_frames: usize, alignment: usize) -> Option<AllocatedFrames> {
    trace!("allocate {} frames", num_frames);
    allocate_frames_deferred(None, Some(alignment), num_frames)
}

/// Allocates frames with no constraints on the starting physical address,
/// with a size given by the number of bytes.
///
/// This function still allocates whole frames by rounding up the number of bytes.
#[allow(unused)]
pub fn allocate_frames_by_bytes(num_bytes: usize) -> Option<AllocatedFrames> {
    allocate_frames_by_bytes_deferred(None, None, num_bytes)
}

/// Allocates frames starting at the given `PAddr` with a size given in number of bytes.
///
/// This function still allocates whole frames by rounding up the number of bytes.
#[allow(unused)]
pub fn allocate_frames_by_bytes_at(paddr: PAddr, num_bytes: usize) -> Option<AllocatedFrames> {
    allocate_frames_by_bytes_deferred(Some(paddr), None, num_bytes)
}

/// Allocates the given number of frames starting at (inclusive of) the frame containing the given `PAddr`.
///
#[allow(unused)]
pub fn allocate_frames_at(paddr: PAddr, num_frames: usize) -> Option<AllocatedFrames> {
    allocate_frames_deferred(Some(paddr), None, num_frames)
}

/// Converts the frame allocator from using static memory (a primitive array) to dynamically-allocated memory.
///
/// Call this function once heap allocation is available.
/// Calling this multiple times is unnecessary but harmless, as it will do nothing after the first invocation.
pub fn convert_to_heap_allocated() {
    FREE_GENERAL_FRAMES_LIST.lock().convert_to_heap_allocated();
}

/// A debugging function used to dump the full internal state of the frame allocator.
pub fn dump_frame_allocator_state() {
    println!("----------------- FREE FRAMES LIST --------------");
    FREE_GENERAL_FRAMES_LIST
        .lock()
        .iter()
        .for_each(|e| println!(" {:?}", e));
    println!("-------------------------------------------------");
}
