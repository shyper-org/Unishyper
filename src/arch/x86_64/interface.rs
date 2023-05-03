use core::mem::size_of;

// use crate::board::BOARD_CORE_NUMBER;
use crate::libs::traits::*;

pub const PHYSICAL_MEMORY_OFFSET: u64 = 0xFFFF_8000_0000_0000;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

pub const MAX_VIRTUAL_ADDRESS: usize = usize::MAX;
pub const MIN_USER_VIRTUAL_ADDRESS: usize = 0x0000_0010_0000_0000;
pub const MAX_USER_VIRTUAL_ADDRESS: usize = 0x0000_007F_FFFF_FFFF;

pub const MAX_PAGE_NUMBER: usize = MAX_VIRTUAL_ADDRESS / PAGE_SIZE;

pub const STACK_SIZE: usize = 2_097_152; // PAGE_SIZE * 512

/// The virtual address offset from which physical memory is mapped, as described in
/// https://os.phil-opp.com/paging-implementation/#map-the-complete-physical-memory
/// It's determined by rboot in rboot.conf.
const PA2KVA: usize = 0xFFFF_8000_0000_0000;
const KVA2PA: usize = 0x0000_7FFF_FFFF_FFFF;

impl Address for usize {
    fn pa2kva(&self) -> usize {
        *self | PA2KVA
    }
    fn kva2pa(&self) -> usize {
        *self | KVA2PA
    }
}

pub type ContextFrame = super::context_frame::X86_64ContextFrame;

// #[allow(unused)]
// pub type PageTable = super::page_table::X86_64PageTable;

pub struct Arch;

impl ArchTrait for Arch {
    fn exception_init() {
        x86_64::instructions::interrupts::disable();
        super::processor::configure();
        super::gdt::add_current_core();
        super::exception::init_idt();
        // x86_64::instructions::interrupts::enable();
        info!("exception init success!");
    }

    fn page_table_init() {
        debug!("init page table for x86_64");
        super::page_table::init();
    }

    fn invalidate_tlb() {}

    fn wait_for_interrupt() {
        x86_64::instructions::hlt()
    }

    fn nop() {
        x86_64::instructions::nop()
    }

    fn fault_address() -> usize {
        0
    }

    #[inline(always)]
    fn core_id() -> usize {
        super::cpu_id()
    }

    fn curent_privilege() -> usize {
        0
    }
}
