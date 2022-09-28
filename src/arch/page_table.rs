use alloc::vec::Vec;
use spin::{Once, Mutex};

use crate::arch::*;
use crate::arch::mm::vm_descriptor::*;
use crate::libs::error::{ERROR_INVARG, ERROR_OOM};
use crate::libs::traits::*;
use crate::mm::frame_allocator;
use crate::mm::frame_allocator::AllocatedFrames;
use crate::mm::interface::{PageTableEntryAttrTrait, PageTableTrait, Error, MapGranularity};
use crate::mm::paging::{Entry, EntryAttribute};
use crate::spinlock::SpinlockIrqSave;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTableEntry(usize);

impl ArchPageTableEntryTrait for Aarch64PageTableEntry {
    fn from_pte(value: usize) -> Self {
        Aarch64PageTableEntry(value)
    }

    fn from_pa(pa: usize) -> Self {
        Aarch64PageTableEntry(pa)
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

    fn entry(&self, index: usize) -> Aarch64PageTableEntry {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        unsafe { Aarch64PageTableEntry((addr as *const usize).read_volatile()) }
    }

    fn set_entry(&self, index: usize, value: Aarch64PageTableEntry) {
        let addr = self.to_kva() + index * MACHINE_SIZE;
        unsafe { (addr as *mut usize).write_volatile(value.0) }
    }

    fn make_table(frame_pa: usize) -> Self {
        Aarch64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame_pa))
    }
}

trait Index {
    fn l1x(&self) -> usize;
    fn l2x(&self) -> usize;
    fn l3x(&self) -> usize;
}

impl Index for usize {
    fn l1x(&self) -> usize {
        (self >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
    fn l2x(&self) -> usize {
        (self >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
    fn l3x(&self) -> usize {
        (self >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
    }
}

impl core::convert::From<Aarch64PageTableEntry> for Entry {
    fn from(u: Aarch64PageTableEntry) -> Self {
        use tock_registers::*;
        let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
        Entry::new(
            EntryAttribute::new(
                reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1)
                    || reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
                reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0)
                    || reg.matches_all(PAGE_DESCRIPTOR::AP::RO_EL1_EL0),
                reg.matches_all(PAGE_DESCRIPTOR::AttrIndx::DEVICE),
                !reg.is_set(PAGE_DESCRIPTOR::PXN),
                !reg.is_set(PAGE_DESCRIPTOR::UXN),
                reg.is_set(PAGE_DESCRIPTOR::COW),
                reg.is_set(PAGE_DESCRIPTOR::LIB),
                !reg.is_set(PAGE_DESCRIPTOR::TYPE),
            ),
            (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT,
        )
    }
}

impl core::convert::From<Entry> for Aarch64PageTableEntry {
    fn from(pte: Entry) -> Self {
        Aarch64PageTableEntry(
            (if pte.attribute().u_shared() {
                PAGE_DESCRIPTOR::LIB::True
            } else {
                PAGE_DESCRIPTOR::LIB::False
            } + if pte.attribute().u_copy_on_write() {
                PAGE_DESCRIPTOR::COW::True
            } else {
                PAGE_DESCRIPTOR::COW::False
            } + if pte.attribute().u_executable() {
                PAGE_DESCRIPTOR::UXN::False
            } else {
                PAGE_DESCRIPTOR::UXN::True
            } + if pte.attribute().k_executable() {
                PAGE_DESCRIPTOR::PXN::False
            } else {
                PAGE_DESCRIPTOR::PXN::True
            } + if pte.attribute().device() {
                PAGE_DESCRIPTOR::SH::OuterShareable + PAGE_DESCRIPTOR::AttrIndx::DEVICE
            } else {
                PAGE_DESCRIPTOR::SH::InnerShareable + PAGE_DESCRIPTOR::AttrIndx::NORMAL
            } + if pte.attribute().writable() && pte.attribute().u_readable() {
                PAGE_DESCRIPTOR::AP::RW_EL1_EL0
            } else if pte.attribute().writable() && !pte.attribute().u_readable() {
                PAGE_DESCRIPTOR::AP::RW_EL1
            } else if !pte.attribute().writable() && pte.attribute().u_readable() {
                PAGE_DESCRIPTOR::AP::RO_EL1_EL0
            } else {
                // if !pte.attr.writable() && !pte.attr.u_readable() {
                PAGE_DESCRIPTOR::AP::RO_EL1
            } + if pte.attribute().block() {
                PAGE_DESCRIPTOR::TYPE::Block
            } else {
                PAGE_DESCRIPTOR::TYPE::Table
            } + PAGE_DESCRIPTOR::VALID::True
                + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64)
                + PAGE_DESCRIPTOR::AF::True)
                .value as usize,
        )
    }
}

#[derive(Debug)]
pub struct Aarch64PageTable {
    directory: AllocatedFrames,
    pages: Mutex<Vec<AllocatedFrames>>,
}

static PAGE_TABLE: Once<SpinlockIrqSave<Aarch64PageTable>> = Once::new();

pub fn page_table() -> &'static SpinlockIrqSave<Aarch64PageTable> {
    PAGE_TABLE.get().unwrap()
}

pub fn init() {
    let pgdir_frame = frame_allocator::allocate_frames(1).unwrap();
    pgdir_frame.start().zero();
    let pgdir_addr = pgdir_frame.start().start_address();
    PAGE_TABLE.call_once(|| SpinlockIrqSave::new(PageTable::new(pgdir_frame)));

    info!("page table init, pgdir frame at {}", pgdir_addr);

    use cortex_a::registers::TTBR0_EL1;
    use tock_registers::interfaces::Writeable;
    // Install page table for user address.
    // TTBR0_EL1.write(TTBR0_EL1::BADDR.val((pgdir_frame.start_address().value() >> 1) as u64));
    TTBR0_EL1.set(pgdir_addr.value() as u64);
    crate::arch::Arch::invalidate_tlb();
}

impl PageTableTrait for Aarch64PageTable {
    fn new(directory: AllocatedFrames) -> Self {
        Aarch64PageTable {
            directory,
            pages: Mutex::new(Vec::new()),
        }
    }

    fn base_pa(&self) -> usize {
        self.directory.start_address().value()
    }

    fn map(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        trace!(
            "page table map va 0x{:016x} pa: 0x{:016x}, directory 0x{:x}",
            va,
            pa,
            self.base_pa()
        );
        let directory = Aarch64PageTableEntry::from_pa(self.base_pa());
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
            l1e = Aarch64PageTableEntry::make_table(frame.start_address().value());
            let mut pages = self.pages.lock();
            pages.push(af);
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
            l2e = Aarch64PageTableEntry::make_table(frame.start_address().value());
            let mut pages = self.pages.lock();
            pages.push(af);
            l1e.set_entry(va.l2x(), l2e);
        }
        l2e.set_entry(va.l3x(), Aarch64PageTableEntry::from(Entry::new(attr, pa)));
        Ok(())
    }

    fn map_2mb(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
        assert!(va % MapGranularity::Page2MB as usize == 0);
        assert!(pa % MapGranularity::Page2MB as usize == 0);
        if !attr.block() {
            warn!("map_2mb: required block attribute");
            return Err(ERROR_INVARG);
        }

        let directory = Aarch64PageTableEntry::from_pa(self.base_pa());
        let mut l1e = directory.entry(va.l1x());

        if !l1e.valid() {
            let af = match frame_allocator::allocate_frames(1) {
                Some(af) => af,
                None => {
                    warn!("map_2mb: failed to allocate one frame for l1e");
                    return Err(ERROR_OOM);
                }
            };
            let frame = af.start().clone();
            frame.zero();
            l1e = Aarch64PageTableEntry::make_table(frame.start_address().value());
            let mut pages = self.pages.lock();
            pages.push(af);
            directory.set_entry(va.l1x(), l1e);
        }
        let l2e = l1e.entry(va.l2x());
        if !l2e.valid() {
            // Map as PTE_BLOCK.
            let entry = Aarch64PageTableEntry::from(Entry::new(attr, pa));
            l1e.set_entry(va.l2x(), entry);
        } else {
            warn!("map_2mb: lvl 2 already mapped with 0x{:x}", l2e.to_pte());
        }
        Ok(())
    }

    fn unmap(&self, va: usize) {
        debug!("unmap va {:x}", va);
        let directory = Aarch64PageTableEntry::from_pa(self.directory.start_address().value());
        let l1e = directory.entry(va.l1x());
        assert!(l1e.valid());
        let l2e = l1e.entry(va.l2x());
        assert!(l2e.valid());
        l2e.set_entry(va.l3x(), Aarch64PageTableEntry(0));
    }

    fn unmap_2mb(&self, va: usize) {
        debug!("unmap_2mb va {:x}", va);
        assert!(va % MapGranularity::Page2MB as usize == 0);
        let directory = Aarch64PageTableEntry::from_pa(self.directory.start_address().value());
        let l1e = directory.entry(va.l1x());
        assert!(l1e.valid());
        l1e.set_entry(va.l2x(), Aarch64PageTableEntry(0));
    }

    fn insert_page(
        &self,
        va: usize,
        user_frame: crate::mm::Frame,
        attr: EntryAttribute,
    ) -> Result<(), Error> {
        let pa = user_frame.start_address().value();
        if let Some(p) = self.lookup_page(va) {
            if p.pa() != pa {
                // replace mapped frame
                self.remove_page(va)?;
            }
        }
        self.map(va, pa, attr)?;
        crate::arch::Arch::invalidate_tlb();
        Ok(())
    }

    fn lookup_entry(&self, va: usize) -> Option<(Entry, MapGranularity)> {
        let directory = Aarch64PageTableEntry::from_pa(self.directory.start_address().value());
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
        let directory = Aarch64PageTableEntry::from_pa(self.directory.start_address().value());
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

    fn remove_page(&self, va: usize) -> Result<(), Error> {
        if let Some(_) = self.lookup_page(va) {
            self.unmap(va);
            // crate::arch::Arch::invalidate_tlb();
            Ok(())
        } else {
            Err(ERROR_INVARG)
        }
    }

    fn recursive_map(&self, va: usize) {
        assert_eq!(va % (1 << PAGE_TABLE_L1_SHIFT), 0);
        let directory = Aarch64PageTableEntry::from_pa(self.directory.start_address().value());
        let l1x = va / (1 << PAGE_TABLE_L1_SHIFT);
        directory.set_entry(
            l1x,
            Aarch64PageTableEntry::from(Entry::new(
                EntryAttribute::user_readonly(),
                self.directory.start_address().value(),
            )),
        );
    }
}
