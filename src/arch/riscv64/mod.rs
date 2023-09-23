mod context_frame;
mod exception;
mod start;
mod vm_descriptor;

pub mod irq;
pub mod page_table;

use core::mem::size_of;
use tock_registers::interfaces::Readable;
use crate::libs::traits::*;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

pub const MAX_VIRTUAL_ADDRESS: usize = usize::MAX;
pub const MIN_USER_VIRTUAL_ADDRESS: usize = 0x0000_0010_0000_0000;
pub const MAX_USER_VIRTUAL_ADDRESS: usize = 0x0000_007F_FFFF_FFFF;

pub const MAX_PAGE_NUMBER: usize = MAX_VIRTUAL_ADDRESS / PAGE_SIZE;

const PA2KVA: usize = 0xFFFF_FFFF_0000_0000;
const KVA2PA: usize = 0xFFFF_FFFF;

impl Address for usize {
    fn pa2kva(&self) -> usize {
        *self | PA2KVA
    }
    fn kva2pa(&self) -> usize {
        *self & KVA2PA
    }
}

pub type ContextFrame = context_frame::Riscv64TrapContextFrame;
pub type ThreadContext = context_frame::ThreadContext;

pub struct Arch;

impl ArchTrait for Arch {
    fn exception_init() {
        exception::init();
    }

    fn page_table_init() {
        page_table::init();
    }

    fn invalidate_tlb() {
        riscv::barrier::sfence_vma_all();
    }

    fn wait_for_interrupt() {
        riscv::asm::wfi();
    }

    fn nop() {
        riscv::asm::nop();
    }

    fn fault_address() -> usize {
        riscv::regs::STVAL.get() as usize
    }

    #[inline(always)]
    fn core_id() -> crate::libs::cpu::CoreId {
        // Note: a pointer to hart_id is stored in sscratch
        riscv::regs::SSCRATCH.get() as usize
        // unsafe { ((riscv::regs::SSCRATCH.get() as usize) as *const usize).read() }
    }

    fn curent_privilege() -> usize {
        0
    }

    fn pop_context_first(ctx: usize) -> ! {
        extern "C" {
            fn _pop_context_first(ctx: usize) -> !;
        }
        unsafe { _pop_context_first(ctx) }
    }

    fn set_thread_id(_tid: u64) {}

    fn get_tls_ptr() -> *const u8 {
        0xDEAD_BEEF as *const u8
    }

    fn set_tls_ptr(_tls_ptr: u64) {}
}
