mod context_frame;
mod exception;
pub mod irq;
mod mmu;
pub mod page_table;
pub mod registers;
pub mod smc;
mod vm_descriptor;

use crate::board::BOARD_CORE_NUMBER;
use crate::libs::traits::*;
use core::mem::size_of;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

pub const MAX_VIRTUAL_ADDRESS: usize = usize::MAX;
pub const MIN_USER_VIRTUAL_ADDRESS: usize = 0x0000_0000_0000_0000;
pub const MAX_USER_VIRTUAL_ADDRESS: usize = 0x0000_007F_FFFF_FFFF;

pub const MAX_PAGE_NUMBER: usize = MAX_VIRTUAL_ADDRESS / PAGE_SIZE;

#[allow(unused)]
pub const KERNEL_STACK_SIZE: usize = 32_768; // // PAGE_SIZE * 8

// pub const STACK_SIZE: usize = 1_048_576; // PAGE_SIZE * 256
pub const STACK_SIZE: usize = 2_097_152; // PAGE_SIZE * 512

const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

impl Address for usize {
    fn pa2kva(&self) -> usize {
        *self | PA2KVA
    }
    fn kva2pa(&self) -> usize {
        *self & KVA2PA
    }
}

pub type ContextFrame = context_frame::TrapContextFrame;
pub type ThreadContext = context_frame::ThreadContext;

use cortex_a::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

pub struct Arch;

impl ArchTrait for Arch {
    fn exception_init() {
        exception::init();
    }

    fn page_table_init() {
        page_table::init();
    }

    fn invalidate_tlb() {
        unsafe {
            core::arch::asm!("dsb ishst", "tlbi vmalle1is", "dsb ish", "isb");
        }
    }

    fn wait_for_interrupt() {
        cortex_a::asm::wfi();
    }

    fn nop() {
        cortex_a::asm::nop();
    }

    fn fault_address() -> usize {
        FAR_EL1.get() as usize
    }

    #[inline(always)]
    fn core_id() -> crate::libs::cpu::CoreId {
        MPIDR_EL1.get() as usize & (BOARD_CORE_NUMBER - 1)
    }

    fn curent_privilege() -> usize {
        (CurrentEL.get() as usize & 0b1100) >> 2
    }

    #[inline(always)]
    fn pop_context_first(ctx: usize) -> ! {
        extern "C" {
            fn _pop_context_first(ctx: usize) -> !;
        }
        unsafe { _pop_context_first(ctx) }
    }

    fn set_thread_id(tid: u64) {
        TPIDRRO_EL0.set(tid);
    }

    fn get_tls_ptr() -> *const u8 {
        TPIDR_EL0.get() as *const u8
    }

    fn set_tls_ptr(tls_ptr: u64) {
        TPIDR_EL0.set(tls_ptr);
    }
}
