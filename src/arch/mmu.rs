use super::mm::vm_descriptor::*;
use tock_registers::interfaces::{ReadWriteable, Writeable};

use super::interface::BOARD_DEVICE_MEMORY_RANGE;
use super::interface::BOARD_NORMAL_MEMORY_RANGE;
use super::interface::PAGE_SHIFT;
use super::interface::PAGE_SIZE;

const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

type PageDirectoryEntry = u64;

#[inline(always)]
fn block_entry(output_addr: usize, device: bool) -> PageDirectoryEntry {
    (PAGE_DESCRIPTOR::PXN::False
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((output_addr >> PAGE_SHIFT) as u64)
        + PAGE_DESCRIPTOR::AF::True
        + PAGE_DESCRIPTOR::AP::RW_EL1
        + PAGE_DESCRIPTOR::TYPE::Block
        + PAGE_DESCRIPTOR::VALID::True
        + if device {
            PAGE_DESCRIPTOR::AttrIndx::DEVICE + PAGE_DESCRIPTOR::SH::OuterShareable
        } else {
            PAGE_DESCRIPTOR::AttrIndx::NORMAL + PAGE_DESCRIPTOR::SH::InnerShareable
        })
    .value
}

#[inline(always)]
const fn invalid_entry() -> PageDirectoryEntry {
    0
}

#[repr(C)]
#[repr(align(4096))]
pub struct PageDirectory([PageDirectoryEntry; ENTRY_PER_PAGE]);

#[no_mangle]
pub unsafe extern "C" fn populate_page_table(pt: &mut PageDirectory) {
    const ONE_GIGABYTE: usize = 0x4000_0000;

    for i in 0..ENTRY_PER_PAGE {
        pt.0[i] = invalid_entry();
    }
    for i in BOARD_DEVICE_MEMORY_RANGE.step_by(ONE_GIGABYTE) {
        // println!("BOARD_DEVICE_MEMORY_RANGE: 0x{:x}",i);00
        pt.0[i / ONE_GIGABYTE] = block_entry(i, true);
    }
    for i in BOARD_NORMAL_MEMORY_RANGE.step_by(ONE_GIGABYTE) {
        // println!("BOARD_NORMAL_MEMORY_RANGE: 0x{:x}",i);
        pt.0[i / ONE_GIGABYTE] = block_entry(i, false);
    }
    // special mapping for kernel elf image
    pt.0[2] = block_entry(0x80000000, false);
}

#[no_mangle]
pub unsafe extern "C" fn mmu_init(pt: &PageDirectory) {
    use cortex_a::registers::*;
    // Memory Attribute Indirection Register
    // Provides the memory attribute encodings corresponding to the possible AttrIndx values in a Long-descriptor format translation table entry for stage 1 translations at EL1.
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck,
    );
    // Translation Table Base Register 0 (EL1)
    // Holds the base address of translation table 0, and information about the memory it occupies.
    // This is one of the translation tables for the stage 1 translation of memory accesses at EL0 and EL1.
    // Translation table base address, bits[47:x].
    TTBR0_EL1.set(&pt.0 as *const _ as u64);
    // Translation Table Base Register 1 (EL1)
    // Holds the base address of translation table 1, and information about the memory it occupies.
    // This is one of the translation tables for the stage 1 translation of memory accesses at EL0 and EL1.
    TTBR1_EL1.set(&pt.0 as *const _ as u64);

    // Translation Control Register
    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
            + TCR_EL1::TBI1::Ignored
            + TCR_EL1::AS::ASID16Bits
            + TCR_EL1::IPS::Bits_44
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::TG1::KiB_4
            + TCR_EL1::SH0::Inner
            + TCR_EL1::SH1::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::EPD1::EnableTTBR1Walks
            + TCR_EL1::A1::TTBR0
            + TCR_EL1::T0SZ.val(64 - 39)
            + TCR_EL1::T1SZ.val(64 - 39),
    );

    use cortex_a::asm::barrier::*;
    isb(SY);
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    isb(SY);
}
