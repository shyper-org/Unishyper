use alloc::vec::Vec;
use spin::{Once, Mutex};

use crate::arch::*;
use super::vm_descriptor::*;
use crate::libs::error::{ERROR_INVARG, ERROR_OOM};
use crate::libs::traits::*;
use crate::mm::frame_allocator;
use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::interface::{PageTableEntryAttrTrait, PageTableTrait, Error, MapGranularity};
use crate::mm::paging::{Entry, EntryAttribute};
use crate::libs::synch::spinlock::SpinlockIrqSave;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct RISCV64PageTableEntry(usize);

impl ArchPageTableEntryTrait for RISCV64PageTableEntry {
    fn from_pte(value: usize) -> Self {
        RISCV64PageTableEntry(value)
    }

    fn from_pa(pa: usize) -> Self {
        RISCV64PageTableEntry((pa >> 12) << 10)
    }

    fn to_pte(&self) -> usize {
        self.0
    }

    fn to_pa(&self) -> usize {
        (self.0 >> 10) << 12
    }

    fn to_kva(&self) -> usize {
        self.to_pa().pa2kva()
    }

    fn valid(&self) -> bool {
        // V and NOT RWX
        self.0 & 0b1 != 0
    }

    fn blocked(&self) -> bool {
        // XWR=000 means this entry pointers to next level of page table.
        self.0 & 0b1110 != 0
    }

    fn entry(&self, index: usize) -> Self {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        // debug!("entry {:#x} index {} addr {:#x}", self.0, index, addr);
        unsafe { RISCV64PageTableEntry((addr as *const usize).read_volatile()) }
    }

    fn set_entry(&self, index: usize, value: Self) {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        // debug!(
        //     "set_entry {:#x} index {} addr {:#x} value {:#x}",
        //     self.0, index, addr, value.0
        // );
        unsafe { (addr as *mut usize).write_volatile(value.0) }
    }

    fn make_table(frame_pa: usize) -> Self {
        RISCV64PageTableEntry(
            (TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_PPN.val((frame_pa >> PAGE_SHIFT) as u64)
                + TABLE_DESCRIPTOR::DIRTY::True
                + TABLE_DESCRIPTOR::ACCESSED::True
                + TABLE_DESCRIPTOR::USER::True
                + TABLE_DESCRIPTOR::VALID::True)
                .value as usize,
        )
    }
}

trait Index {
    fn l1x(&self) -> usize;
    fn l2x(&self) -> usize;
    fn l3x(&self) -> usize;
}

impl Index for usize {
    fn l1x(&self) -> usize {
        self >> PAGE_TABLE_L1_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
    fn l2x(&self) -> usize {
        self >> PAGE_TABLE_L2_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
    fn l3x(&self) -> usize {
        self >> PAGE_TABLE_L3_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
}

impl core::convert::From<RISCV64PageTableEntry> for Entry {
    fn from(u: RISCV64PageTableEntry) -> Self {
        use tock_registers::*;
        let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
        Entry::new(
            EntryAttribute::new(
                reg.is_set(PAGE_DESCRIPTOR::W),
                reg.is_set(PAGE_DESCRIPTOR::USER),
                false, // riscv do not has bits indicating device memory
                false, // reg.is_set(PAGE_DESCRIPTOR::X)  && SUM bit in sstatus
                reg.is_set(PAGE_DESCRIPTOR::X),
                reg.is_set(PAGE_DESCRIPTOR::COW),
                reg.is_set(PAGE_DESCRIPTOR::LIB),
                false,
            ),
            (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT,
        )
    }
}

impl core::convert::From<Entry> for RISCV64PageTableEntry {
    fn from(pte: Entry) -> Self {
        let r = RISCV64PageTableEntry(
            (if pte.attribute().u_shared() {
                PAGE_DESCRIPTOR::LIB::True
            } else {
                PAGE_DESCRIPTOR::LIB::False
            } + if pte.attribute().u_copy_on_write() {
                PAGE_DESCRIPTOR::COW::True
            } else {
                PAGE_DESCRIPTOR::COW::False
            } + if pte.attribute().u_executable() {
                PAGE_DESCRIPTOR::X::True
            } else {
                PAGE_DESCRIPTOR::X::False
            } + if pte.attribute().u_readable() {
                PAGE_DESCRIPTOR::R::True
            } else {
                PAGE_DESCRIPTOR::R::False
            } + if pte.attribute().writable() {
                PAGE_DESCRIPTOR::W::True
            } else {
                PAGE_DESCRIPTOR::W::False
            } + PAGE_DESCRIPTOR::DIRTY::True
                + PAGE_DESCRIPTOR::ACCESSED::True
                + PAGE_DESCRIPTOR::VALID::True
                + PAGE_DESCRIPTOR::USER::True
                + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64))
            .value as usize,
        );
        r
    }
}

#[derive(Debug)]
pub struct RISCV64PageTable {
    directory_entry: RISCV64PageTableEntry,
    frames: Mutex<Vec<AllocatedFrames>>,
}

static PAGE_TABLE: Once<SpinlockIrqSave<RISCV64PageTable>> = Once::new();

pub fn page_table() -> &'static SpinlockIrqSave<RISCV64PageTable> {
    PAGE_TABLE.get().unwrap()
}

pub fn init() {
    PAGE_TABLE.call_once(|| {
        extern "C" {
            // Note: link-time label, see linker.ld
            fn KERNEL_PAGE_DIRECTORY();
        }
        let dir_entry = RISCV64PageTableEntry::from_pa((KERNEL_PAGE_DIRECTORY as usize).kva2pa());
        // debug!("page_table init entry at {:#x}", dir_entry.0);
        SpinlockIrqSave::new(RISCV64PageTable {
            directory_entry: dir_entry,
            frames: Mutex::new(Vec::new()),
        })
    });
}

impl PageTableTrait for RISCV64PageTable {
    fn base_pa(&self) -> usize {
        self.directory_entry.0
    }

    fn map(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        trace!(
            "page table map va 0x{:016x} pa: 0x{:016x}, directory 0x{:x}",
            va,
            pa,
            self.base_pa()
        );
        let directory = self.directory_entry;
        let mut l1e = directory.entry(va.l1x());
        if !l1e.valid() {
            let af = match frame_allocator::allocate_frames(1) {
                Some(af) => af,
                None => {
                    warn!("map: failed to allocate one frame for l1e");
                    return Err(ERROR_OOM);
                }
            };
            let frame = af.start().clone();
            frame.zero();
            l1e = RISCV64PageTableEntry::make_table(frame.start_address().value());
            let mut frames = self.frames.lock();
            frames.push(af);
            directory.set_entry(va.l1x(), l1e);
        }
        let mut l2e = l1e.entry(va.l2x());
        if !l2e.valid() {
            let af = match frame_allocator::allocate_frames(1) {
                Some(af) => af,
                None => {
                    warn!("map: failed to allocate one frame for l2e");
                    return Err(ERROR_OOM);
                }
            };
            let frame = af.start().clone();
            frame.zero();
            l2e = RISCV64PageTableEntry::make_table(frame.start_address().value());
            let mut frames = self.frames.lock();
            frames.push(af);
            l1e.set_entry(va.l2x(), l2e);
        }
        l2e.set_entry(va.l3x(), RISCV64PageTableEntry::from(Entry::new(attr, pa)));
        Ok(())
    }

    fn map_2mb(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        assert!(va % MapGranularity::Page2MB as usize == 0);
        assert!(pa % MapGranularity::Page2MB as usize == 0);
        trace!(
            "page table map_2mb va 0x{:016x} pa: 0x{:016x}, directory 0x{:x}",
            va,
            pa,
            self.base_pa()
        );
        if !attr.block() {
            warn!("map_2mb: required block attribute");
            return Err(ERROR_INVARG);
        }
        let directory = self.directory_entry;
        let mut l1e = directory.entry(va.l1x());
        if !l1e.valid() {
            let af = match frame_allocator::allocate_frames(1) {
                Some(af) => af,
                None => {
                    warn!("map: failed to allocate one frame for l1e");
                    return Err(ERROR_OOM);
                }
            };
            let frame = af.start().clone();
            frame.zero();
            l1e = RISCV64PageTableEntry::make_table(frame.start_address().value());
            let mut frames = self.frames.lock();
            frames.push(af);
            directory.set_entry(va.l1x(), l1e);
        }
        let l2e = l1e.entry(va.l2x());
        if !l2e.valid() {
            // 2MB Mapped.
            let entry = RISCV64PageTableEntry::from(Entry::new(attr, pa));
            l1e.set_entry(va.l2x(), entry);
        } else {
            warn!("map_2mb: lvl 2 already mapped with 0x{:x}", l2e.to_pte());
        }
        Ok(())
    }

    fn unmap(&mut self, va: usize) {
        trace!("unmap va {:x}", va);
        let directory = self.directory_entry;
        let l1e = directory.entry(va.l1x());
        assert!(l1e.valid());
        let l2e = l1e.entry(va.l2x());
        assert!(l2e.valid());
        l2e.set_entry(va.l3x(), RISCV64PageTableEntry(0));
    }

    fn unmap_2mb(&mut self, va: usize) {
        trace!("unmap_2mb va {:x}", va);
        assert!(va % MapGranularity::Page2MB as usize == 0);
        let directory = self.directory_entry;
        let l1e = directory.entry(va.l1x());
        assert!(l1e.valid());
        l1e.set_entry(va.l2x(), RISCV64PageTableEntry(0));
    }

    fn lookup_entry(&self, va: usize) -> Option<(Entry, MapGranularity)> {
        let directory = self.directory_entry;
        let l1e = directory.entry(va.l1x());
        if !l1e.valid() {
            return None;
        }
        let l2e = l1e.entry(va.l2x());
        if !l2e.valid() {
            return None;
        }
        if l2e.blocked() {
            return Some((Entry::from(l2e), MapGranularity::Page2MB));
        }
        let l3e = l2e.entry(va.l3x());
        if l3e.valid() {
            return Some((Entry::from(l3e), MapGranularity::Page4KB));
        } else {
            return None;
        }
    }

    fn lookup_page(&self, va: usize) -> Option<Entry> {
        let directory = self.directory_entry;
        let l1e = directory.entry(va.l1x());
        if !l1e.valid() {
            return None;
        }
        let l2e = l1e.entry(va.l2x());
        if !l2e.valid() {
            return None;
        }
        let l3e = l2e.entry(va.l3x());
        if l3e.valid() {
            Some(Entry::from(l3e))
        } else {
            None
        }
    }
}
