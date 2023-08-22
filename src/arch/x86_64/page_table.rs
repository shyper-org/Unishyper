use alloc::collections::BTreeMap;
use spin::Once;

use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::Size2MiB;
use x86_64::structures::paging::{
    frame::PhysFrame as Frame,
    mapper::OffsetPageTable,
    Mapper,
    page::{Page, Size4KiB},
    page_table::{PageTable as x86PageTable, PageTableFlags},
    FrameAllocator, FrameDeallocator,
};

use crate::arch::x86_64::{MACHINE_SIZE, PHYSICAL_MEMORY_OFFSET, PAGE_SIZE};
use crate::libs::error::{ERROR_INVARG, ERROR_INTERNAL};
use crate::libs::traits::*;
use crate::mm::frame_allocator;
use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::interface::{PageTableEntryAttrTrait, PageTableTrait, Error, MapGranularity};
#[cfg(feature = "mpk")]
use crate::mm::interface::PageTableEntryAttrZoneTrait;
use crate::mm::paging::{Entry, EntryAttribute};
use crate::libs::synch::spinlock::SpinlockIrqSave;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
#[allow(unused)]
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct X86_64PageTableEntry(usize);

#[allow(unused)]
impl ArchPageTableEntryTrait for X86_64PageTableEntry {
    fn from_pte(value: usize) -> Self {
        X86_64PageTableEntry(value)
    }

    fn from_pa(pa: usize) -> Self {
        X86_64PageTableEntry(pa)
    }

    fn to_pte(&self) -> usize {
        self.0
    }

    fn to_pa(&self) -> usize {
        self.0 & 0x0000_FFFF_FFFF_F000
    }

    fn to_kva(&self) -> usize {
        self.to_pa().pa2kva()
    }

    fn valid(&self) -> bool {
        self.0 & 0b11 != 0
    }

    fn blocked(&self) -> bool {
        self.0 & 0b01 != 0
    }

    fn entry(&self, index: usize) -> X86_64PageTableEntry {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        unsafe { X86_64PageTableEntry((addr as *const usize).read_volatile()) }
    }

    fn set_entry(&self, index: usize, value: X86_64PageTableEntry) {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        unsafe { (addr as *mut usize).write_volatile(value.0) }
    }

    fn make_table(frame_pa: usize) -> Self {
        X86_64PageTableEntry(0)
        // X86_64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame_pa))
    }
}

#[derive(Debug)]
pub struct X86_64PageTable {
    page_table: OffsetPageTable<'static>,
    // pages: Mutex<Vec<AllocatedFrames>>,
    dir_frame: Frame,
}

static PAGE_TABLE: Once<SpinlockIrqSave<X86_64PageTable>> = Once::new();

fn frame_to_page_table(frame: Frame) -> *mut x86PageTable {
    let vaddr = (frame.start_address().as_u64() as usize).pa2kva();
    // debug!(
    //     "frame_to_page_table , frame {:#?}, page_table at {:#x}",
    //     frame, vaddr
    // );
    vaddr as *mut x86PageTable
}

pub fn page_table() -> &'static SpinlockIrqSave<X86_64PageTable> {
    PAGE_TABLE.get().unwrap()
}

pub fn init() {
    let frame = Cr3::read().0;
    debug!("page table init, frame {:#?}", frame);
    let table = unsafe { &mut *frame_to_page_table(frame) };

    for (_l4_idx, l4_entry) in table.iter_mut().enumerate() {
        if !l4_entry.is_unused() {
            l4_entry.set_flags(l4_entry.flags() | PageTableFlags::USER_ACCESSIBLE);
            debug!("page table dir entry {} flags {:?}", _l4_idx, l4_entry);
            let l3_page_table = unsafe { &mut *frame_to_page_table(l4_entry.frame().unwrap()) };
            for (_l3_idx, l3_entry) in l3_page_table.iter_mut().enumerate() {
                if !l3_entry.is_unused() {
                    l3_entry.set_flags(l3_entry.flags() | PageTableFlags::USER_ACCESSIBLE);

                    if l3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        continue;
                    }

                    let l2_page_table =
                        unsafe { &mut *frame_to_page_table(l3_entry.frame().unwrap()) };
                    for (_l2_idx, l2_entry) in l2_page_table.iter_mut().enumerate() {
                        if !l2_entry.is_unused() {
                            l2_entry.set_flags(l2_entry.flags() | PageTableFlags::USER_ACCESSIBLE);

                            if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                                // debug!(
                                //     "[{}][{}] l2 page table entry {} map huge page, continue",
                                //     idx, l3_idx, l2_idx,
                                // );
                                continue;
                            }
                            // let l1_page_table =
                            //     unsafe { &mut *frame_to_page_table(l2_entry.frame().unwrap()) };
                            // for (_l1_idx, l1_entry) in l1_page_table.iter_mut().enumerate() {
                            //     if !l1_entry.is_unused()
                            //         && l1_entry.flags().contains(PageTableFlags::NO_EXECUTE)
                            //     // && l1_entry.flags().contains(PageTableFlags::ACCESSED)
                            //     {
                            //         // l1_entry.set_flags(
                            //         //     l1_entry.flags() | PageTableFlags::USER_ACCESSIBLE,
                            //         // );
                            //         println!(
                            //             "[{}][{}][{}][{}] l1 vaddr {:#x}'s entry {:?}",
                            //             _l4_idx,
                            //             _l3_idx,
                            //             _l2_idx,
                            //             _l1_idx,
                            //             0xffff << 12 << 9 << 9 << 9 << 9
                            //                 | _l4_idx << 12 << 9 << 9 << 9
                            //                 | _l3_idx << 12 << 9 << 9
                            //                 | _l2_idx << 12 << 9
                            //                 | _l1_idx << 12,
                            //             l1_entry
                            //         );
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }

    // let demo_vaddr = 0xffffff000001951a as u64;
    // let page_4kb = Page::<Size4KiB>::containing_address(VirtAddr::new(demo_vaddr));
    // let l4_index = page_4kb.p4_index();
    // let l3_index = page_4kb.p3_index();
    // let l2_index = page_4kb.p2_index();
    // let l1_index = page_4kb.p1_index();
    // println!("==============================================================");
    // debug!(
    //     "demo_vaddr {:?} l4_index {:?} l3_index {:?} l2_index {:?} l1_index {:?}",
    //     page_4kb,
    //     usize::from(l4_index),
    //     usize::from(l3_index),
    //     usize::from(l2_index),
    //     usize::from(l1_index),
    // );

    let physical_memory_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET);
    PAGE_TABLE.call_once(|| {
        SpinlockIrqSave::new(X86_64PageTable {
            page_table: unsafe { OffsetPageTable::new(table, physical_memory_offset) },
            // pages: Mutex::new(Vec::new()),
            dir_frame: frame,
        })
    });
    debug!(
        "Page table init ok, dir at {:#x}",
        page_table().lock().base_pa()
    );

    page_table().lock().init_main_zone_flags();
}

/// Todo: this seems awkward.
static mut ALLOCATOR_FRAME_MAP: BTreeMap<u64, AllocatedFrames> = BTreeMap::new();

struct FrameAllocatorForX86;

unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorForX86 {
    fn allocate_frame(&mut self) -> Option<Frame> {
        frame_allocator::allocate_frames(1).map(|allocated_frames| {
            let frame_addr = allocated_frames.start_address().value() as u64;
            debug!(
                "Pagetable FrameAllocatorForX86 alloc frame on {:#x}",
                frame_addr
            );
            unsafe {
                ALLOCATOR_FRAME_MAP.insert(frame_addr, allocated_frames);
            }
            Frame::containing_address(PhysAddr::new(frame_addr))
        })
    }
}

impl FrameDeallocator<Size4KiB> for FrameAllocatorForX86 {
    unsafe fn deallocate_frame(&mut self, frame: Frame) {
        let frame_addr = frame.start_address().as_u64();
        debug!(
            "Pagetable FrameAllocatorForX86 dealloc frame on {:#x}",
            frame_addr
        );
        match ALLOCATOR_FRAME_MAP.remove(&frame_addr) {
            Some(_allocated_frame) => {}
            None => warn!(
                "FrameAllocatorForX86 deallocate_frame frame {:#x} not exist",
                frame_addr
            ),
        }
    }
}

#[allow(unused)]
impl X86_64PageTable {
    pub fn dump_entry_flags_of_va(&mut self, va: usize) {
        let page_4kb = Page::<Size4KiB>::containing_address(VirtAddr::new(va as u64));
        let l4_index = page_4kb.p4_index();
        let l3_index = page_4kb.p3_index();
        let l2_index = page_4kb.p2_index();
        let l1_index = page_4kb.p1_index();
        let l4_entry = &self.page_table.level_4_table()[usize::from(l4_index)];
        let l3_page_table = unsafe { &*frame_to_page_table(l4_entry.frame().unwrap()) };
        let l3_entry = &l3_page_table[l3_index];
        let l2_page_table = unsafe { &*frame_to_page_table(l3_entry.frame().unwrap()) };
        let l2_entry = &l2_page_table[l2_index];
        if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            println!(
                "va {:#x} mapped as 2MB Huge page with flags {:?}",
                va,
                l2_entry.flags()
            );
            return;
        }
        let l1_page_table = unsafe { &*frame_to_page_table(l2_entry.frame().unwrap()) };
        let l1_entry = &l1_page_table[l1_index];
        println!(
            "va {:#x} mapped as 4KB page with flags {:?}",
            va,
            l1_entry.flags()
        );
    }

    fn init_main_zone_flags(&mut self) {
        extern "C" {
            fn PROTECTED_DATA_START();
            fn PROTECTED_DATA_END();
        }
        let protected_data_start = PROTECTED_DATA_START as usize;
        let protected_data_end = PROTECTED_DATA_END as usize;

        info!(
            "Protected data region [{:#x} to {:#x}]",
            protected_data_start, protected_data_end
        );

        assert_eq!(protected_data_start % PAGE_SIZE, 0);
        assert_eq!(protected_data_end % PAGE_SIZE, 0);

        for va in (protected_data_start..protected_data_end).step_by(PAGE_SIZE) {
            let page_4kb = Page::<Size4KiB>::containing_address(VirtAddr::new(va as u64));
            let l4_index = page_4kb.p4_index();
            let l3_index = page_4kb.p3_index();
            let l2_index = page_4kb.p2_index();
            let l1_index = page_4kb.p1_index();
            let l4_entry = &self.page_table.level_4_table()[usize::from(l4_index)];
            debug!(
                "get l4 pte of index {}, {:?}",
                usize::from(l4_index),
                l4_entry.flags()
            );
            let l3_page_table = unsafe { &*frame_to_page_table(l4_entry.frame().unwrap()) };
            let l3_entry = &l3_page_table[l3_index];
            debug!(
                "get l3 pte of index {}, {:?}",
                usize::from(l3_index),
                l3_entry.flags()
            );
            let l2_page_table = unsafe { &*frame_to_page_table(l3_entry.frame().unwrap()) };
            let l2_entry = &l2_page_table[l2_index];
            debug!(
                "get l2 pte of index {}, {:?}",
                usize::from(l2_index),
                l2_entry.flags()
            );
            if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                panic!("va {:#x} mapped as 2MB Huge page", va);
                return;
            }
            let l1_page_table = unsafe { &mut *frame_to_page_table(l2_entry.frame().unwrap()) };
            let mut l1_entry = &mut l1_page_table[l1_index];
            debug!(
                "get l1 pte of index {}, {:?}",
                usize::from(l1_index),
                l1_entry.flags()
            );
            l1_entry.set_flags(
                l1_entry.flags() | PageTableFlags::BIT_59 | PageTableFlags::USER_ACCESSIBLE,
            );
            debug!("va {:#x} mapped as flags {:?}", va, l1_entry.flags());
            println!("==============================================================");
        }
        x86_64::instructions::tlb::flush_all();
    }

    pub fn dump_entry(&mut self, va: usize) {
        let page_4kb = Page::<Size4KiB>::containing_address(VirtAddr::new(va as u64));
        let l4_index = page_4kb.p4_index();
        let l3_index = page_4kb.p3_index();
        let l2_index = page_4kb.p2_index();
        let l1_index = page_4kb.p1_index();
        println!("==============================================================");
        debug!(
            "l4_index {:?} l3_index {:?} l2_index {:?} l1_index {:?}",
            usize::from(l4_index),
            usize::from(l3_index),
            usize::from(l2_index),
            usize::from(l1_index),
        );
        let l4_entry = &self.page_table.level_4_table()[usize::from(l4_index)];
        debug!(
            "get l4 pte of index {}, {:?}",
            usize::from(l4_index),
            l4_entry.flags()
        );
        let l3_page_table = unsafe { &*frame_to_page_table(l4_entry.frame().unwrap()) };
        let l3_entry = &l3_page_table[l3_index];
        debug!(
            "get l3 pte of index {}, {:?}",
            usize::from(l3_index),
            l3_entry.flags()
        );
        let l2_page_table = unsafe { &*frame_to_page_table(l3_entry.frame().unwrap()) };
        let l2_entry = &l2_page_table[l2_index];
        debug!(
            "get l2 pte of index {}, {:?}",
            usize::from(l2_index),
            l2_entry.flags()
        );
        if l2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            debug!("va {:#x} mapped as 2MB Huge page", va);
            return;
        }
        let l1_page_table = unsafe { &*frame_to_page_table(l2_entry.frame().unwrap()) };
        let l1_entry = &l1_page_table[l1_index];
        debug!(
            "get l1 pte of index {}, {:?}",
            usize::from(l1_index),
            l1_entry.flags()
        );
        let frame = self.page_table.translate_page(page_4kb.clone());
        debug!("after map_4kb , get frame {:?}", frame);
        println!("==============================================================");
    }

    fn dump_entry_2mb(&mut self, va: usize) {
        let page_2mb = Page::<Size2MiB>::containing_address(VirtAddr::new(va as u64));
        let l4_index = page_2mb.p4_index();
        let l3_index = page_2mb.p3_index();
        let l2_index = page_2mb.p2_index();
        println!("==============================================================");
        debug!(
            "l4_index {:?} l3_index {:?} l2_index {:?}",
            usize::from(l4_index),
            usize::from(l3_index),
            usize::from(l2_index)
        );
        let l4_entry = &self.page_table.level_4_table()[usize::from(l4_index)];
        debug!(
            "get l4 pte of index {}, {:?}",
            usize::from(l4_index),
            l4_entry.flags()
        );
        let l3_page_table = unsafe { &*frame_to_page_table(l4_entry.frame().unwrap()) };
        let l3_entry = &l3_page_table[l3_index];
        debug!(
            "get l3 pte of index {}, {:?}",
            usize::from(l3_index),
            l3_entry.flags()
        );
        let l2_page_table = unsafe { &*frame_to_page_table(l3_entry.frame().unwrap()) };
        let l2_entry = &l2_page_table[l2_index];
        debug!(
            "get l2 pte of index {}, {:?}",
            usize::from(l2_index),
            l2_entry.flags()
        );
        let frame = self.page_table.translate_page(page_2mb.clone());
        debug!("after map_2mb , get frame {:?}", frame);
        println!("==============================================================");
    }
}

// Todo：remove redundant functions, not fully implemented yet!!!
impl PageTableTrait for X86_64PageTable {
    fn base_pa(&self) -> usize {
        self.dir_frame.start_address().as_u64() as usize
    }

    fn map(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        // let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if attr.writable() {
            flags |= PageTableFlags::WRITABLE;
        }
        if attr.device() {
            flags |= PageTableFlags::NO_CACHE;
        }
        if !(attr.k_executable() && attr.u_executable()) {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        #[cfg(feature = "mpk")]
        {
            // (the protection key located in bits 62:59 of the paging-structure entry that mapped the page containing the linear address.
            let zone_id = attr.get_zone_id();

            if zone_id & 1 == 1 {
                flags |= PageTableFlags::BIT_59;
            }
            if zone_id & 2 == 2 {
                flags |= PageTableFlags::BIT_60;
            }
            if zone_id & 4 == 4 {
                flags |= PageTableFlags::BIT_61;
            }
            if zone_id & 8 == 8 {
                flags |= PageTableFlags::BIT_62;
            }
        }

        debug!(
            "page table map va 0x{:016x} pa: 0x{:016x}, flags {:?}",
            va,
            pa,
            flags.clone()
        );

        let page_4kb = Page::<Size4KiB>::containing_address(VirtAddr::new(va as u64));
        let frame_4kb = Frame::<Size4KiB>::containing_address(PhysAddr::new(pa as u64));
        match unsafe {
            self.page_table
                .map_to(page_4kb, frame_4kb, flags, &mut FrameAllocatorForX86)
        } {
            Ok(mapper_flush) => {
                mapper_flush.flush();
            }
            Err(err) => {
                warn!(
                    "x86_64 map 4KB page {:#x} failed of pa {:#x}, err {:?}",
                    va, pa, err
                );
                return Err(ERROR_INTERNAL);
            }
        }
        // self.dump_entry(va);
        Ok(())
    }

    fn map_2mb(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        assert!(va % MapGranularity::Page2MB as usize == 0);
        assert!(pa % MapGranularity::Page2MB as usize == 0);
        if !attr.block() {
            warn!("map_2mb: required block attribute");
            return Err(ERROR_INVARG);
        }

        let mut flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;

        // MPK only works for user pages.
        flags |= PageTableFlags::USER_ACCESSIBLE;

        #[cfg(feature = "mpk")]
        {
            let zone_id = attr.get_zone_id();
            // (the protection key located in bits 62:59 of the paging-structure entry that mapped the page containing the linear address.
            if zone_id & 1 == 1 {
                flags |= PageTableFlags::BIT_59;
            }
            if zone_id & 2 == 2 {
                flags |= PageTableFlags::BIT_60;
            }
            if zone_id & 4 == 4 {
                flags |= PageTableFlags::BIT_61;
            }
            if zone_id & 8 == 8 {
                flags |= PageTableFlags::BIT_62;
            }
        }

        debug!(
            "page table map_2mb va 0x{:016x} pa: 0x{:016x}, flags {:?}",
            va,
            pa,
            flags.clone()
        );

        let page_2mb = Page::<Size2MiB>::containing_address(VirtAddr::new(va as u64));
        let frame_2mb = Frame::<Size2MiB>::containing_address(PhysAddr::new(pa as u64));
        match unsafe {
            self.page_table
                .map_to(page_2mb, frame_2mb, flags, &mut FrameAllocatorForX86)
        } {
            Ok(mapper_flush) => {
                mapper_flush.flush();
            }
            Err(err) => {
                warn!(
                    "x86_64 map 2MB page {:#x} failed of pa {:#x}, err {:?}",
                    va, pa, err
                );
                return Err(ERROR_INTERNAL);
            }
        }
        // self.dump_entry_2mb(va);
        Ok(())
    }

    fn unmap(&mut self, va: usize) {
        debug!("unmap va {:x}", va);
        self.page_table
            .unmap(Page::<Size4KiB>::containing_address(VirtAddr::new(
                va as u64,
            )))
            .unwrap()
            .1
            .flush();
    }

    fn unmap_2mb(&mut self, va: usize) {
        debug!("unmap_2mb va {:x}", va);
        assert!(va % MapGranularity::Page2MB as usize == 0);
        self.page_table
            .unmap(Page::<Size2MiB>::containing_address(VirtAddr::new(
                va as u64,
            )))
            .unwrap()
            .1
            .flush();
    }

    // fn insert_page(
    //     &self,
    //     va: usize,
    //     user_frame: crate::mm::Frame,
    //     attr: EntryAttribute,
    // ) -> Result<(), Error> {
    //     Ok(())
    // }

    fn lookup_entry(&self, _va: usize) -> Option<(Entry, MapGranularity)> {
        None
    }

    fn lookup_page(&self, _va: usize) -> Option<Entry> {
        None
    }

    // fn remove_page(&self, va: usize) -> Result<(), Error> {
    //     if let Some(_) = self.lookup_page(va) {
    //         self.unmap(va);
    //         // crate::arch::Arch::invalidate_tlb();
    //         Ok(())
    //     } else {
    //         Err(ERROR_INVARG)
    //     }
    // }

    // fn recursive_map(&self, va: usize) {
    //     assert_eq!(va % (1 << PAGE_TABLE_L1_SHIFT), 0);
    // }
}
